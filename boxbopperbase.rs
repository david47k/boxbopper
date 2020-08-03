// Box Bopper: Sokoban clone in rust


// wasm boilerplate
mod utils;
// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

use wasm_bindgen::prelude::*;
use js_sys::{Array,JsString};

#[cfg(target_arch = "wasm32")]
use web_sys::console;
#[cfg(not(target_arch = "wasm32"))]
use std::time::{SystemTime,UNIX_EPOCH};

pub mod builtins;
use builtins::BUILTIN_LEVELS;

pub mod vector;
use vector::{Vector,Move,ALLMOVES};

pub mod level;
use level::{Level,load_builtin};

pub mod dgens;
use dgens::{contains_only};

// we need time in msec since unix epoch (for js compatibility)
#[cfg(not(target_arch = "wasm32"))]
pub fn get_time_ms() -> f64 {
    let since_the_epoch = SystemTime::now()
        .duration_since(UNIX_EPOCH)
		.expect("Time went backwards");
	let ms: u64 = since_the_epoch.as_secs() * 1000 + since_the_epoch.subsec_nanos() as u64 / 1_000_000;
	let lopart: u32 = (ms & 0xFFFFFFFF) as u32; 
	let hipart: u32 = (ms >> 32) as u32;
	let t: f64 = f64::from(lopart) + (f64::from(hipart) * 4.294967296e9);
	t
}

#[cfg(target_arch = "wasm32")]
pub fn get_time_ms() -> f64 {
	let t = (js_sys::Date::now() as u64 / 10) * 10;
	return t as f64;
}

#[cfg(not(target_arch = "wasm32"))]
pub fn console_log(s: &str) {
	println!("{}",s);
}

#[cfg(target_arch = "wasm32")]
pub fn console_log(s: &str) {
	console::log(&Array::from(&JsValue::from(s)));
}


#[wasm_bindgen]
#[derive(Clone, Copy, PartialEq, PartialOrd, Eq)]
pub enum Obj { Wall=0, Space=1, Boulder=2, Hole=3, Human=4, HumanInHole=5, BoulderInHole=6 }

impl Obj {
	pub fn to_char(&self) -> char {
		match self {
			Obj::Wall => '#',
			Obj::Space => ' ',
			Obj::Boulder => '*',
			Obj::Hole => 'O',
			Obj::Human => '&',
			Obj::HumanInHole => '%',
			Obj::BoulderInHole => '@',
		}
	}

	pub fn from_char(c: &char) -> Obj {
		return match c {
			'#' => Obj::Wall,
			' ' => Obj::Space,
			'*' => Obj::Boulder,
			'O' => Obj::Hole,
			'&' => Obj::Human,
			'%' => Obj::HumanInHole,
			'@' => Obj::BoulderInHole,
			_ => panic!("Char does not represent a valid object"),
		};
	}
}


#[wasm_bindgen]
pub struct Game {
	pub level_number: u32,
	pub num_moves: u32,
	move_history: Vec::<Move>,
	pub human_pos: Vector,
	level: Level,
	sprites: Vec::<Sprite>,
	move_queue: Vec::<Move>,
}


impl Game {			// non-js
	pub fn get_move_options(&self) -> Vec<Move> {
		let mut options: Vec<Move> = Vec::with_capacity(4);
		for movedir in ALLMOVES.iter() {
			let hp = self.human_pos;
			match self.get_object_at_point(&hp.add(&movedir.to_vector())) {
				Obj::Space | Obj::Hole => {
					options.push(*movedir);
				}
				Obj::Boulder | Obj::BoulderInHole => { 
					// What's past the boulder? We can push into Space and Hole, nothing else.
					//match get_object_in_dir(self, &hp, &d.double()) {
					match self.get_object_at_point(&hp.add(&movedir.to_vector().double())) {
						Obj::Space | Obj::Hole => { options.push(*movedir); },
						_ => {}				
					}
				}
				_ => {}
			};
		}	
		options
	}
	pub fn append_move(&mut self, _move : &Move) {
		self.move_queue.insert(0, *_move);
	}
	pub fn process_moves(&mut self)  {
		if self.move_queue.len() == 0 { 
			return; 
		}
		if self.sprites[0].is_moving() { 
			return;
		}			
		
		let	_move = self.move_queue.pop().unwrap();
		console_log("move popped from queue");

		// check it is a valid option
		if !self.get_move_options().contains(&_move) {
			return;
		}

		// add to history
		self.move_history.push(_move);
		self.num_moves += 1;

		// remove old human
		let idx = self.human_pos.to_index(&self.level.w);
		let human_obj = self.level.get_obj_at_idx(idx);
		let new_obj = match human_obj {
			Obj::Human => { Obj::Space },
			Obj::HumanInHole => { Obj::Hole },
			_ => { panic!("Human not in tracked location!"); }
		};
		self.level.set_obj_at_idx(idx, new_obj);
		
		// new human point
		let np = self.human_pos.add(&_move.to_vector());
		let idx = np.to_index(&self.level.w);	
		
		// check destination point
		let obj = self.level.get_obj_at_idx(idx);
		let new_obj = match obj {
			Obj::Space => { Obj::Human },
			Obj::Hole  => { Obj::HumanInHole },
			Obj::Boulder | Obj::BoulderInHole => {  
				// Move boulder in to next square
				let boulder_pt = &self.human_pos.add(&_move.to_vector().double());
				let i = boulder_pt.to_index(&self.level.w);
				let o = self.level.get_obj_at_idx(i);
				if o == Obj::Hole {
					self.level.set_obj_at_idx(i, Obj::BoulderInHole);
				} else {
					self.level.set_obj_at_idx(i, Obj::Boulder);
				}
			
				// We pushed the boulder
				if obj == Obj::BoulderInHole {
					Obj::HumanInHole
				} else {
					Obj::Human
				}
			},
			_ => { panic!("Human not allowed there!"); }
		};

		// set up a visual transition for this move
		// locate sprite human (always first one in the vec), replace sprite times + xy coordinates
		
		let trans = Trans {
			initial_xy: self.human_pos.clone(),
			final_xy: np.clone(),
			duration: 100_f64,
		};
		self.sprites[0].apply_trans(trans);

		// place human
		self.level.set_obj_at_idx(idx, new_obj);	
		self.human_pos = np;		
	}

}


#[wasm_bindgen]
impl Game {
	#[wasm_bindgen(constructor)]
	pub fn new(mut levelnum: u32) -> Game {
		// restarts the game, using builtin levels
		utils::set_panic_hook();
		if levelnum as usize >= BUILTIN_LEVELS.len() {
			levelnum = (BUILTIN_LEVELS.len()-1) as u32;
		}
		let base_level = load_builtin(levelnum as usize).unwrap(); 
		return Game::new_from_level(&base_level,levelnum);
	}

	pub fn new_from_level(base_level: &Level, levelnum: u32) -> Game {
		// restarts the game, using what's in base_level
		let mut sp = Vec::with_capacity(32);
		sp.push(Sprite::new(0, Obj::Human, get_time_ms(), 0.0, base_level.human_pos, base_level.human_pos));
		Game {
			level_number: levelnum,
			num_moves: 0,
			move_history: Vec::<Move>::new(),
			human_pos: base_level.human_pos.clone(),
			level: base_level.clone(),
			sprites: sp,
			move_queue: Vec::<Move>::new(),
		}
	}

	pub fn process_moves_js(&mut self) {
		self.process_moves();
	}

	pub fn get_max_level_number(&self) -> u32 {
		(BUILTIN_LEVELS.len() - 1) as u32
	}

	pub fn process_keys(&mut self, keys: Array) {
		// which keys are currently held down
		if keys.length()==1 {
			match keys.get(0).as_f64().unwrap() as u32 {		// or something like .get(0).dyn_into::<u32>.unwrap()
				87 | 38 => self.append_move(&Move::Up),
			 	68 | 39 => self.append_move(&Move::Right),
			 	83 | 40 => self.append_move(&Move::Down),
			 	65 | 37 => self.append_move(&Move::Left),
				_       => {},
			}
		}
	}

	pub fn get_move_history(&self) -> Array {
		self.move_history.clone().into_iter().map(|m| JsValue::from(m as u32)).collect()
	}	

	
	pub fn display(&self) {
		println!("--------------------------------------------------------------------------------");
		println!("{} moves: {}", self.num_moves, moves_to_string(&self.move_history));
		println!();
		// print level
		for y in 0..self.level.h {
			for x in 0..self.level.w {
				print!("{}",&self.level.get_obj_at_idx(y * self.level.w + x).to_char());
			}
			println!();
		}
		println!();
	}
	
	pub fn get_object_at_point(&self, point: &Vector) -> Obj {
		// bounds check point
		if point.to_usize().0 >= self.level.w || point.to_usize().1 >= self.level.h {
			return Obj::Wall;
		}
		
		// fetch object
		return self.level.get_obj_at_pt(point);
	}


	pub fn get_move_options_js(&self) -> Array {
		self.get_move_options().clone().into_iter().map(|m| JsValue::from(m as u32)).collect()
	}	

	pub fn append_move_js(&mut self, _move: u32) {
		let result = Move::from_u32(_move);
		match result {
			Some(m) => self.append_move(&m),
			None  => {},
		}
	}

	// As Level isn't available from JS (no copy implemetor), we have these functions to access the level
	pub fn get_level_title(&self) -> JsString {
		self.level.get_title()
	}
	pub fn get_level_width(&self) -> u32 {
		self.level.w as u32
	}
	pub fn get_level_height(&self) -> u32 {
		self.level.h as u32
	}
	pub fn have_win_condition(&self) -> bool {
		self.level.have_win_condition()
	}
	pub fn get_level_data(&self) -> Array {
		self.level.get_data()
	}

	pub fn get_sprites_js(&mut self) -> Array {
		// return all the sprites, with their up-to-date-coordinates, as type SpriteInfo
		self.sprites.clone().into_iter().map(|mut s| JsValue::from(s.get_sprite_info())).collect()
	}

	pub fn get_sprites_debug(&mut self) -> Array {
		self.sprites.clone().into_iter().map(|s| JsValue::from(s)).collect()
	}
}




pub fn moves_to_string(moves: &Vec::<Move>) -> String {
	let mut s: String = "".to_string();
	for m in moves.iter() {
		s = s + &m.to_string();
	}
	return s;
}




#[wasm_bindgen]
#[derive(Clone,Copy)]
pub struct SpriteInfo {		// location information passed back to JS so it can render the sprite in the correct location
	pub id: u32,
	pub obj: Obj,
	pub x: f64,
	pub y: f64,
}

#[wasm_bindgen]
#[derive(Clone,Copy)]
pub struct Trans {
	pub initial_xy: Vector,
	pub final_xy: Vector,
	pub duration: f64,
}

#[wasm_bindgen]
#[derive(Clone)]
pub struct Sprite {
	pub id: u32,	// unique id for objects of this Obj type. i.e. playerNumber or boulderNumber.
	pub obj: Obj,	// what type of object affects how we render it
	pub initial_xy: Vector,		// movement transition to apply
	pub initial_time: f64,
	pub final_xy: Vector,
	pub duration: f64,
	priv_is_moving: u32,
}

impl Sprite {
	pub fn new(id: u32, obj: Obj, initial_time: f64, duration: f64, initial_xy: Vector, final_xy: Vector) -> Sprite {
		Sprite {
			id,
			obj,
			initial_time,
			duration,
			initial_xy,
			final_xy,
			priv_is_moving: 0,
		}
	}
	pub fn apply_trans(&mut self, trans: Trans) {		
		if !self.is_moving() { 
			self.initial_time = get_time_ms();
			self.duration = trans.duration;
			self.initial_xy = trans.initial_xy.clone();
			self.final_xy = trans.final_xy.clone();
		} else {
			// ignore the requested movement !
			console_log("move requested while already moving!");
		}
	}
	pub fn is_moving(&self) -> bool {
		get_time_ms() < (self.initial_time + self.duration)
	}

	pub fn get_xy(&mut self) -> [f64;2] {
		// linear
		let t = get_time_ms();
		if t <= self.initial_time {
			return [f64::from(self.initial_xy.0), f64::from(self.initial_xy.1)];
		} 
		if t >= (self.initial_time + self.duration) {
			// update location
			// self.initial_xy = Vector(self.final_xy.0, self.final_xy.1);

			return [self.final_xy.0.into(), self.final_xy.1.into()];
		}
		// according to time & duration, we are currently moving
		let mut delta: f64 = (t - self.initial_time) / self.duration;
		if delta > 1_f64 {
			delta = 1_f64;
			self.initial_xy = self.final_xy.clone();
			console_log("post movement 2");
		}
		let nx = delta * f64::from(self.final_xy.0 - self.initial_xy.0) + f64::from(self.initial_xy.0);
		let ny = delta * f64::from(self.final_xy.1 - self.initial_xy.1) + f64::from(self.initial_xy.1);
		[nx,ny]
	}
	pub fn get_sprite_info(&mut self) -> SpriteInfo {
		let pt = self.get_xy();
		SpriteInfo {
			id: self.id,
			obj: self.obj,
			x: pt[0],
			y: pt[1],
		}
	}
}


fn _undo_move(base_level: Level, our_state: &mut Game)  {
	// resets to a certain spot using move history
	our_state.num_moves = 0;
	our_state.move_history = Vec::<Move>::new();
	our_state.level = base_level;
}


fn _apply_moves(state: &mut Game, moves: &Vec::<Move>) -> Result<(),String> {
	let _s = state;
	let _m = moves;
	return Err("Not implemented".to_string());
}



