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

pub mod builtins;
use builtins::{BUILTIN_LEVELS};

pub mod vector;
use vector::{Vector,Move,ALLMOVES};

pub mod level;
use level::{Level};

pub mod dgens;
//use dgens::{contains_only};

pub mod sprite;
use sprite::{Sprite,Trans};

pub mod time;
use time::{get_time_ms};

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
pub enum Obj { Wall=0, Space=1, Boxx=2, Hole=3, Human=4, HumanInHole=5, BoxxInHole=6 }

impl Obj {
	pub fn to_char(&self) -> char {
		match self {
			Obj::Wall => '#',
			Obj::Space => ' ',
			Obj::Boxx => '*',
			Obj::Hole => 'O',
			Obj::Human => '&',
			Obj::HumanInHole => '%',
			Obj::BoxxInHole => '@',
		}
	}

	pub fn from_char(c: &char) -> Obj {
		return match c {
			'#' => Obj::Wall,
			' ' => Obj::Space,
			'*' => Obj::Boxx,
			'O' => Obj::Hole,
			'&' => Obj::Human,
			'%' => Obj::HumanInHole,
			'@' => Obj::BoxxInHole,
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
				Obj::Boxx | Obj::BoxxInHole => { 
					// What's past the boxx? We can push into Space and Hole, nothing else.
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
		let idx = self.human_pos.to_index(self.level.w);
		let human_obj = self.level.get_obj_at_idx(idx);
		let new_obj = match human_obj {
			Obj::Human => { Obj::Space },
			Obj::HumanInHole => { Obj::Hole },
			_ => { panic!("Human not in tracked location!"); }
		};
		self.level.set_obj_at_idx(idx, new_obj);
		
		// new human point
		let np = self.human_pos.add(&_move.to_vector());
		let idx = np.to_index(self.level.w );	
		
		// check destination point
		let obj = self.level.get_obj_at_idx(idx);
		let mut moved_boxx = false;
		let new_obj = match obj {
			Obj::Space => { Obj::Human },
			Obj::Hole  => { Obj::HumanInHole },
			Obj::Boxx | Obj::BoxxInHole => {  
				// Move boxx in to next square
				moved_boxx = true;
				let boxx_pt = &self.human_pos.add(&_move.to_vector().double());
				let i = boxx_pt.to_index(self.level.w);
				let o = self.level.get_obj_at_idx(i);
				if o == Obj::Hole {
					self.level.set_obj_at_idx(i, Obj::BoxxInHole);
				} else {
					self.level.set_obj_at_idx(i, Obj::Boxx);
				}
			
				// We pushed the boxx
				if obj == Obj::BoxxInHole {
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

		// if we moved the boxx, we got to figure out which boxx it is, and update the sprite with a trans
		if moved_boxx {
			let initial_boxx_pt = &np;
			let final_boxx_pt = &self.human_pos.add(&_move.to_vector().double());
			let mut i = 0;
			while i < self.sprites.len()-1 {
				i += 1;
				if self.sprites[i].obj == Obj::Boxx && self.sprites[i].final_xy == *initial_boxx_pt {
					// move this one
					let trans = Trans {
						initial_xy: initial_boxx_pt.clone(),
						final_xy: final_boxx_pt.clone(),
						duration: 100_f64,
					};
					self.sprites[i].apply_trans(trans);
					break;
				}
			}
		}

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
		let base_level = Level::from_builtin(levelnum as usize).unwrap(); 
		return Game::new_from_level(&base_level,levelnum);
	}

	pub fn new_from_level(base_level: &Level, levelnum: u32) -> Game {
		// restarts the game, using what's in base_level
		let mut sp = Vec::with_capacity(32);
		sp.push(Sprite::new(0, Obj::Human, get_time_ms(), 0.0, base_level.human_pos, base_level.human_pos));
		base_level.get_boxx_pts().iter().enumerate().for_each(|(n,b)| sp.push(Sprite::new(n as u32, Obj::Boxx, get_time_ms(), 0.0, *b, *b)));
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
		println!("{}", self.level.to_string());
		println!();
	}
	
	pub fn get_object_at_point(&self, point: &Vector) -> Obj {
		// bounds check point
		if point.to_usize().0 >= self.level.w as usize || point.to_usize().1 >= self.level.h as usize {
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
