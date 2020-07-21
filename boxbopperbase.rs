
// Box Bopper: Sokoban clone in rust


// wasm boilerplate

mod utils;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

use js_sys::Array;

// use

use std::io::{BufReader,BufRead};
use std::fs::File;
use std::convert::TryInto;
use wasm_bindgen::prelude::*;

mod builtins;
use builtins::BUILTIN_LEVELS;


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
	fn to_usize(&self) -> (usize,usize) {
		(self.0 as usize, self.1 as usize)
	}
}


#[wasm_bindgen]
#[derive(Clone, Copy, PartialEq)]
pub enum Move { Up, Right, Down, Left }

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
	pub fn from_u32(n: u32) -> Move {
		match n {
			0 => Move::Up,
			1 => Move::Right,
			2 => Move::Down,
			3 => Move::Left,
			_ => panic!("invalid u32 in Move::from_u32"),
		}
	}
}

pub const ALLMOVES: [Move; 4] = [ Move::Up, Move::Right, Move::Down, Move::Left ];


#[wasm_bindgen]
#[derive(Clone, Copy, PartialEq)]
pub enum Object { Wall, Space, Boulder, Hole, Human, HumanInHole, BoulderInHole }

impl Object {
	pub fn to_char(&self) -> char {
		match self {
			Object::Wall => '#',
			Object::Space => ' ',
			Object::Boulder => '*',
			Object::Hole => 'O',
			Object::Human => '&',
			Object::HumanInHole => '%',
			Object::BoulderInHole => '@',
		}
	}
}


#[wasm_bindgen]
#[derive(Clone)]
pub struct Level {
	title: String,
	w: usize,
	h: usize,
	human_pos: Vector,
	data: Vec::<Object>,
}

#[wasm_bindgen]
impl Level {
	pub fn get_data(&self) -> Array {
		self.data.clone().into_iter().map(|m| JsValue::from(m as u32)).collect()
	}
}

#[wasm_bindgen]
pub struct Game {
	num_moves: u32,
	move_history: Vec::<Move>,
	human_pos: Vector,
	level: Level,
}

// These ones are not accessible to js
impl Game {
	pub fn get_move_options(&self) -> Vec<Move> {
		let mut options: Vec<Move> = Vec::new();
		for movedir in ALLMOVES.iter() {
			let hp = self.human_pos;
			match self.get_object_at_point(&hp.add(&movedir.to_vector())) {
				Object::Space | Object::Hole => {
					options.push(*movedir);
				}
				Object::Boulder | Object::BoulderInHole => { 
					// What's past the boulder? We can push into Space and Hole, nothing else.
					//match get_object_in_dir(self, &hp, &d.double()) {
					match self.get_object_at_point(&hp.add(&movedir.to_vector().double())) {
						Object::Space | Object::Hole => { options.push(*movedir); },
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
			Object::Human => { Object::Space },
			Object::HumanInHole => { Object::Hole },
			_ => { panic!("Human not in tracked location!"); }
		};
		self.level.data[idx] = new_obj;
		
		// new human point
		let np = self.human_pos.add(&_move.to_vector());
		let idx = np.to_index(&self.level.w);	
		
		// check destination point
		let obj = self.level.data[idx];
		let new_obj = match obj {
			Object::Space => { Object::Human },
			Object::Hole  => { Object::HumanInHole },
			Object::Boulder | Object::BoulderInHole => {  
				// Move boulder in to next square
				let boulder_pt = &self.human_pos.add(&_move.to_vector().double());
				let i = boulder_pt.to_index(&self.level.w);
				let o = self.level.data[i];
				if o == Object::Hole {
					self.level.data[i] = Object::BoulderInHole;
				} else {
					self.level.data[i] = Object::Boulder;
				}
			
				// We pushed the boulder
				if obj == Object::BoulderInHole {
					Object::HumanInHole
				} else {
					Object::Human
				}
			},
			_ => { panic!("Human not allowed there!"); }
		};

		// place human
		self.level.data[idx] = new_obj;	
		self.human_pos = np;		
	}
}

#[wasm_bindgen]
impl Game {
	#[wasm_bindgen(constructor)]
	pub fn new(base_level: &Level) -> Game {
		// restarts the game, using what's in base_level
		Game {
			num_moves: 0,
			move_history: Vec::<Move>::new(),
			human_pos: base_level.human_pos.clone(),
			level: Level {
				title: base_level.title.clone(),
				w: base_level.w,
				h: base_level.h,
				human_pos: base_level.human_pos.clone(),
				data: base_level.data.clone(),
			}
		}
	}
	
	pub fn get_move_history(&self) -> Array {
		self.move_history.clone().into_iter().map(|m| JsValue::from(m as u32)).collect()
	}	

	pub fn have_win_condition(&self) -> bool {
		for obj in self.level.data.iter() {
			match obj {
				Object::Boulder | Object::Hole | Object::HumanInHole => return false,
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
	
	pub fn get_object_at_point(&self, point: &Vector) -> Object {
		// bounds check point
		if point.to_usize().0 >= self.level.w || point.to_usize().1 >= self.level.h {
			return Object::Wall;
		}
		
		// fetch object
		return self.level.data[point.to_index(&self.level.w)];
	}


	pub fn get_move_options_js(&self) -> Array {
		self.get_move_options().clone().into_iter().map(|m| JsValue::from(m as u32)).collect()
	}	

	pub fn apply_move_js(&mut self, _move: u32) {
		self.apply_move(&Move::from_u32(_move));
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
	let mut data = Vec::<Object>::new();
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
	let mut data = Vec::<Object>::new();
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
				if c == '&' || c == '@' {
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


pub fn char_to_obj(c: &char) -> Result<Object,String> {
	return match c {
		'#' => Ok(Object::Wall),
		' ' => Ok(Object::Space),
		'*' => Ok(Object::Boulder),
		'O' => Ok(Object::Hole),
		'&' => Ok(Object::Human),
		'%' => Ok(Object::HumanInHole),
		'@' => Ok(Object::BoulderInHole),
		_ => Err("Char does not represent a valid object".to_string()),
	};
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



