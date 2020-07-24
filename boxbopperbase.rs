// Box Bopper: Sokoban clone in rust

// wasm boilerplate
mod utils;
// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
use js_sys::{Array,JsString};

// use
use std::io::{BufReader,BufRead};
use std::fs::File;
use std::convert::TryInto;
use std::time::{SystemTime,UNIX_EPOCH};
use wasm_bindgen::prelude::*;
mod builtins;
use builtins::BUILTIN_LEVELS;

// we need time in msec since unix epoch (for js compatibility)
pub fn _get_time_rust() -> u64 {
    let since_the_epoch = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
	let ms = since_the_epoch.as_secs() * 1000 +
			since_the_epoch.subsec_nanos() as u64 / 1_000_000;
	ms
}

// A point and a direction can both be implemented as a Vector
#[wasm_bindgen]
#[derive(Clone, Copy)]
pub struct Vector (i32,i32);

#[wasm_bindgen]
impl Vector {
	#[wasm_bindgen(constructor)]
	pub fn new(x: i32, y: i32) -> Vector {
		Vector(x,y)
	}

	pub fn add(&self, dir: &Vector) -> Vector {
		Vector(self.0 + dir.0, self.1 + dir.1)
	}
	fn to_index(&self, width: &usize) -> usize {
		width * (self.1 as usize) + (self.0 as usize)
	}
	pub fn double(&self) -> Vector {
		Vector(self.0 * 2, self.1 * 2)
	}
	pub fn scale_by(&self, n: i32) -> Vector {
		Vector(self.0 * n, self.1 * n)
	}
	pub fn as_array(&self) -> Array {
		[ self.0, self.1 ].iter().map(|m| JsValue::from(*m)).collect()
	}
	fn to_usize(&self) -> (usize,usize) {
		(self.0 as usize, self.1 as usize)
	}
}


#[wasm_bindgen]
#[derive(Clone, Copy, PartialEq)]
pub enum Move { Up=0, Right=1, Down=2, Left=3 }

impl Move {
	pub fn to_vector(&self) -> Vector {
		match self {
			Move::Up    => Vector( 0, -1 ),
			Move::Right => Vector( 1,  0 ),
			Move::Down  => Vector( 0,  1 ),
			Move::Left  => Vector(-1,  0 ),
		}
	}
	pub fn to_string(&self) -> String {
		match self {
			Move::Up    => String::from("U"),
			Move::Right => String::from("R"),
			Move::Down  => String::from("D"),
			Move::Left  => String::from("L"),
		}
	}
	pub fn from_u32(n: u32) -> Option<Move> {
		match n {
			0 => Some(Move::Up),
			1 => Some(Move::Right),
			2 => Some(Move::Down),
			3 => Some(Move::Left),
			_ => None,
		}
	}
}

pub const ALLMOVES: [Move; 4] = [ Move::Up, Move::Right, Move::Down, Move::Left ];


#[wasm_bindgen]
#[derive(Clone, Copy, PartialEq)]
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
}

pub fn get_time_ms() -> f64 {
	let t = js_sys::Date::now();
	return t as f64;
}

#[wasm_bindgen]
#[derive(Clone)]
pub struct Level {
	title: String,
	w: usize,
	h: usize,
	human_pos: Vector,
	data: Vec::<Obj>,
}


#[wasm_bindgen]
pub struct Game {
	pub level_number: u32,
	pub num_moves: u32,
	move_history: Vec::<Move>,
	pub human_pos: Vector,
	level: Level,
	transitions: Vec::<Transition>,
}

// These ones are not accessible to js
impl Game {
	pub fn get_move_options(&self) -> Vec<Move> {
		let mut options: Vec<Move> = Vec::new();
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
	
	pub fn apply_move(&mut self, _move: &Move)  {

		// check it is a valid option
		if !self.get_move_options().contains(&*_move) {
			return;
		}

		// add to history
		self.move_history.push(*_move);
		self.num_moves += 1;

		// remove old human
		let idx = self.human_pos.to_index(&self.level.w);
		let human_obj = self.level.data[idx];
		let new_obj = match human_obj {
			Obj::Human => { Obj::Space },
			Obj::HumanInHole => { Obj::Hole },
			_ => { panic!("Human not in tracked location!"); }
		};
		self.level.data[idx] = new_obj;
		
		// new human point
		let np = self.human_pos.add(&_move.to_vector());
		let idx = np.to_index(&self.level.w);	
		
		// check destination point
		let obj = self.level.data[idx];
		let new_obj = match obj {
			Obj::Space => { Obj::Human },
			Obj::Hole  => { Obj::HumanInHole },
			Obj::Boulder | Obj::BoulderInHole => {  
				// Move boulder in to next square
				let boulder_pt = &self.human_pos.add(&_move.to_vector().double());
				let i = boulder_pt.to_index(&self.level.w);
				let o = self.level.data[i];
				if o == Obj::Hole {
					self.level.data[i] = Obj::BoulderInHole;
				} else {
					self.level.data[i] = Obj::Boulder;
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
		self.transitions.push(Transition::new(Obj::Human,self.human_pos,get_time_ms(),np,get_time_ms()+500_f64));

		// place human
		self.level.data[idx] = new_obj;	
		self.human_pos = np;		
	}
}

// these ones are accessible via js
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
		Game {
			level_number: levelnum,
			num_moves: 0,
			move_history: Vec::<Move>::new(),
			human_pos: base_level.human_pos.clone(),
			level: Level {
				title: base_level.title.clone(),
				w: base_level.w,
				h: base_level.h,
				human_pos: base_level.human_pos.clone(),
				data: base_level.data.clone(),
			},
			transitions: Vec::new(),
		}
	}

	pub fn get_level_data(&self) -> Array {
		self.level.data.clone().into_iter().map(|obj| JsValue::from(obj as u32)).collect()
	}
	
	pub fn get_level_width(&self) -> u32 {
		self.level.w as u32
	}
	
	pub fn get_level_height(&self) -> u32 {
		self.level.h as u32
	}
	pub fn get_level_title(&self) -> JsString {
		return JsString::from(self.level.title.as_str());
	}

	pub fn get_max_level_number(&self) -> u32 {
		(BUILTIN_LEVELS.len() - 1) as u32
	}

	pub fn process_keys(&mut self, keys: Array) {
		// which keys are currently held down
		if keys.length()==1 {
			match keys.get(0).as_f64().unwrap() as u32 {		// or something like .get(0).dyn_into::<u32>.unwrap()
				87 | 38 => self.apply_move(&Move::Up),
			 	68 | 39 => self.apply_move(&Move::Right),
			 	83 | 40 => self.apply_move(&Move::Down),
			 	65 | 37 => self.apply_move(&Move::Left),
				_       => {},
			}
		}
	}

	pub fn get_move_history(&self) -> Array {
		self.move_history.clone().into_iter().map(|m| JsValue::from(m as u32)).collect()
	}	

	pub fn have_win_condition(&self) -> bool {
		for obj in self.level.data.iter() {
			match obj {
				Obj::Boulder | Obj::Hole | Obj::HumanInHole => return false,
				_ => {},
			};
		}
		return true;
	}
	
	pub fn display(&self) {
		println!("--------------------------------------------------------------------------------");
		println!("{} moves: {}", self.num_moves, moves_to_string(&self.move_history));
		println!();
		// print level
		for y in 0..self.level.h {
			for x in 0..self.level.w {
				print!("{}",&self.level.data[y * self.level.w + x].to_char());
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
		return self.level.data[point.to_index(&self.level.w)];
	}


	pub fn get_move_options_js(&self) -> Array {
		self.get_move_options().clone().into_iter().map(|m| JsValue::from(m as u32)).collect()
	}	

	pub fn apply_move_js(&mut self, _move: u32) {
		let result = Move::from_u32(_move);
		match result {
			Some(m) => self.apply_move(&m),
			None  => {},
		}
	}

	pub fn get_transitions_js(&mut self) -> Array {
		// for each transition, delete it if it's finished, return an array of
		// TransitionInfo for processing in js
		let t = get_time_ms();
		let mut tinfos = Vec::new();
		// theres gotta be a neater way of removing vecs based on vec content
		let mut i: usize = 0;
		while i < self.transitions.len() {
			let trans = self.transitions[i];
			if trans.is_finished(t) {
				self.transitions.remove(i);
				continue; // don't want to increment i as it'll be something else now
			}
			tinfos.push(trans.get_transition_info(t));
			i+=1;
		}
		tinfos.clone().into_iter().map(|m| JsValue::from(m)).collect()
	}
}



pub fn load_level(filename: &str) -> Result<Level,String> {
    let input = match File::open(filename) {
		Ok(x) => x,
		_ => panic!("Failed to open level file: {}", filename),
	};
    let buffered = BufReader::new(input);
	let mut count: usize = 0;
	let mut w = 0;
	let mut data = Vec::<Obj>::new();
	let mut human_pos: Vector = Vector(0,0);

    for line in buffered.lines() {		
		let txt: String = match line {
			Ok(o) => o,
			_ => panic!("Failed to read line from level file."),
		};
        if count == 0 {
			// read in length
			w = txt.len();			
		}
		// check length equal to w
		if txt.len() == w {	
			// split line into characters
			for (i,c) in txt.char_indices() {		// chars() is iterator
				if c == '&' || c == '@' {
					// found human_pos
					human_pos = Vector(i.try_into().unwrap(),count.try_into().unwrap());
				}
				data.push( char_to_obj(&c)? );
			}
		} else {
			break;
		}
		
		count = count + 1;
    }
	let h  = count;

	println!("Dimensions: {} x {}", w, h);
	
	if human_pos.0 == 0 || human_pos.1 == 0 {
		panic!("Human not found in level");
	}
	
	println!("Human at: {}, {}", human_pos.0, human_pos.1);
	
	// my_vec.push(Wall);
	if w < 3 || h < 3 {
		panic!("Width and Height must be >= 3");
	}
	
	let level = Level {
		title: String::from("Untitled"),
		w: w,
		h: h,
		human_pos: human_pos,
		data: data,
	};
	
	return Ok(level);
}


#[wasm_bindgen]
pub fn load_builtin(number: usize) -> Option<Level> {
	// locate string
	if number >= BUILTIN_LEVELS.len() {
		return None;
	}
	
	let level = BUILTIN_LEVELS[number];
	
	// process string
	let mut count: usize = 0;
	let mut w = 0;
	let mut data = Vec::<Obj>::new();
	let mut human_pos: Vector = Vector(0,0);
	let mut level_title: String = String::from("Untitled");

    for line in level.lines() {		
		let txt = line.to_string();
		//println!("{} : {} : {}",txt,line, count);
		if count == 0 {
			// read in title of level
			level_title = txt;
			count += 1;
			continue;
		}
        if count == 1 {
			// read in width of level
			w = txt.len();			
		}
		// check length equal to w
		if txt.len() == w {	
			// split line into characters
			for (i,c) in txt.char_indices() {		// chars() is iterator
				if c == '&' || c == '%' {
					// found human_pos
					human_pos = Vector(i.try_into().unwrap(),(count-1).try_into().unwrap());
				}
				data.push( char_to_obj(&c).unwrap() );
			}
		} else {
			break;
		}
		
		count = count + 1;
    }
	let h  = count - 1;

	println!("Dimensions: {} x {}", w, h);
	
	if human_pos.0 == 0 || human_pos.1 == 0 {
		panic!("Human not found in level");
	}
	
	println!("Human at: {}, {}", human_pos.0, human_pos.1);
	
	if w < 3 || h < 3 {
		panic!("Width and Height must be >= 3");
	}
	
	let level = Level {
		title: level_title,
		w: w,
		h: h,
		human_pos: human_pos,
		data: data,
	};
	
	return Some(level);
}



pub fn moves_to_string(moves: &Vec::<Move>) -> String {
	let mut s: String = "".to_string();
	for m in moves.iter() {
		s = s + &m.to_string();
	}
	return s;
}


pub fn char_to_obj(c: &char) -> Result<Obj,String> {
	return match c {
		'#' => Ok(Obj::Wall),
		' ' => Ok(Obj::Space),
		'*' => Ok(Obj::Boulder),
		'O' => Ok(Obj::Hole),
		'&' => Ok(Obj::Human),
		'%' => Ok(Obj::HumanInHole),
		'@' => Ok(Obj::BoulderInHole),
		_ => Err("Char does not represent a valid object".to_string()),
	};
}


// enum TransitionType { LinearMove = 0}
// Don't need Transition in JS, we'll just use TransitionInfo when we get
#[derive(Clone,Copy)]
pub struct Transition {
	obj: Obj,
//	type: TransitionType,
	initial_xy: Vector,
	initial_time: f64,
	final_xy: Vector,
	final_time: f64,
}

#[wasm_bindgen]
#[derive(Clone,Copy)]
pub struct TransitionInfo {
	pub obj: Obj,
	pub x: f64,
	pub y: f64,
}

impl Transition {
	pub fn new(obj: Obj, initial_xy: Vector, initial_time: f64, final_xy: Vector, final_time: f64) -> Transition {
		Transition {
			obj,
			initial_xy,
			initial_time,
			final_xy,
			final_time,
		}
	}
	pub fn get_point(&self, t: f64) -> [f64;2] {
		// linear
		if t <= self.initial_time {
			return [f64::from(self.initial_xy.0), f64::from(self.initial_xy.1)];
		} else if t>=self.final_time {
			return [self.final_xy.0.into(), self.final_xy.1.into()];
		}
		let delta: f64 = (t - self.initial_time) / (self.final_time - self.initial_time);
		let nx = delta * f64::from(self.final_xy.0 - self.initial_xy.0) + f64::from(self.initial_xy.0);
		let ny = delta * f64::from(self.final_xy.1 - self.initial_xy.1) + f64::from(self.initial_xy.1);
		[nx,ny]
	}
	pub fn get_transition_info(&self, t: f64) -> TransitionInfo {
		let pt = self.get_point(t);
		TransitionInfo {
			obj: self.obj,
			x: pt[0],
			y: pt[1],
		}
	}
	pub fn is_finished(&self, t: f64) -> bool {
		t >= self.final_time
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



