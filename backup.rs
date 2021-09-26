// Box Bopper: Sokoban-like game
// Copyright David Atkinson 2020-2021

use std::io;
use std::io::{BufReader,BufRead};
use std::fs::File;

#[derive(Clone, Copy, PartialEq)]
enum Move { North, East, South, West }

impl Move {
	fn to_dir(&self) -> Direction {
		match self {
			Move::North => NORTH,
			Move::East  => EAST,
			Move::South => SOUTH,
			Move::West  => WEST,
		}
	}
	fn to_string(&self) -> String {
		match self {
			Move::North => String::from("n"),
			Move::East  => String::from("e"),
			Move::South => String::from("s"),
			Move::West  => String::from("w"),
		}
	}
}

const ALLMOVES: [Move; 4] = [ Move::North, Move::East, Move::South, Move::West ];


#[derive(Clone, Copy, PartialEq)]
enum Object { Wall, Space, Boulder, Hole, Human, HumanInHole, BoulderInHole }

impl Object {
	fn to_char(&self) -> char {
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


struct Direction (i32,i32);

impl Direction {
	fn double(&self) -> Direction {
		Direction(self.0 * 2, self.1 * 2)
	}
}

const NORTH: Direction = Direction( 0, -1 );
const EAST: Direction  = Direction( 1,  0 );
const SOUTH: Direction = Direction( 0,  1 );
const WEST: Direction  = Direction(-1,  0 );
const ALLDIRS: [Direction; 4] = [ NORTH, EAST, SOUTH, WEST ];


#[derive(Clone, Copy)]
struct Point (usize,usize);

impl Point {
	fn apply_dir(&self, dir: &Direction) -> Point {
		Point((self.0 as i32 + dir.0) as usize, (self.1 as i32 + dir.1) as usize)
	}
	fn to_index(&self, width: &usize) -> usize {
		width * self.1 + self.0
	}
}


#[derive(Clone)]
struct Level {
	w: usize,
	h: usize,
	human_pos: Point,
	data: Vec::<Object>,
}



struct State {
	num_moves: u32,
	move_history: Vec::<Move>,
	human_pos: Point,
	level: Level,
}

impl State {
	fn have_win_condition(&self) -> bool {
		for obj in self.level.data.iter() {
			match obj {
				Object::Boulder | Object::Hole | Object::HumanInHole => return false,
				_ => {},
			};
		}
		return true;
	}
	
	fn display(&self) {
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
	
	fn get_object_at_point(&self, point: &Point) -> Object {
		// bounds check point
		if point.0 >= self.level.w || point.1 >= self.level.h {
			return Object::Wall;
		}
		
		// fetch object
		return self.level.data[self.level.w * point.1 + point.0];
	}

	fn get_move_options(&self) -> Vec<Move> {
		let mut options: Vec<Move> = [].to_vec();
		for i in 0..4 {
			let d = &ALLDIRS[i];
			let hp = self.human_pos;
			//match get_object_in_dir(self, &hp, d) {
			match self.get_object_at_point(&hp.apply_dir(d)) {
				Object::Space | Object::Hole => {
					options.push(ALLMOVES[i]);
				}
				Object::Boulder | Object::BoulderInHole => { 
					// What's past the boulder? We can push into Space and Hole, nothing else.
					//match get_object_in_dir(self, &hp, &d.double()) {
					match self.get_object_at_point(&hp.apply_dir(&d.double())) {
						Object::Space | Object::Hole => { options.push(ALLMOVES[i]); },
						_ => {}				
					}
				}
				_ => {}
			};
		}	
		options
	}
	
	fn apply_move(&mut self, _move: &Move)  {

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
		let np = self.human_pos.apply_dir(&_move.to_dir());
		let idx = np.to_index(&self.level.w);	
		
		// check destination point
		let obj = self.level.data[idx];
		let new_obj = match obj {
			Object::Space => { Object::Human },
			Object::Hole  => { Object::HumanInHole },
			Object::Boulder | Object::BoulderInHole => {  
				// Move boulder in to next square
				let boulder_pt = &self.human_pos.apply_dir(&_move.to_dir().double());
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


fn load_level(filename: &str) -> Result<Level,String> {
    let input = match File::open(filename) {
		Ok(x) => x,
		_ => panic!("Failed to open level file: {}", filename),
	};
    let buffered = BufReader::new(input);
	let mut count = 0;
	let mut w = 0;
	let mut data = Vec::<Object>::new();
	let mut human_pos: Point = Point(0,0);

	println!("Reading level...");
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
					human_pos = Point(i,count);
				}
				data.push( char_to_obj(&c)? );
			}
		} else {
			break;
		}
		
		count = count + 1;
    }
	let h  = count;

	println!("Width:   {}", w);
	println!("Height:  {}", h);
	
	if human_pos.0 == 0 || human_pos.1 == 0 {
		panic!("Human not found in level");
	}
	
	println!("Human X: {}", human_pos.0);
	println!("Human Y: {}", human_pos.1);
	
	// my_vec.push(Wall);
	if w < 3 || h < 3 {
		panic!("Width and Height must be >= 3");
	}
	
	let level = Level {
		w: w,
		h: h,
		human_pos: human_pos,
		data: data,
	};
	
	return Ok(level);
}


fn moves_to_string(moves: &Vec::<Move>) -> String {
	let mut s: String = "".to_string();
	for m in moves.iter() {
		s = s + &m.to_string();
	}
	return s;
}


fn char_to_obj(c: &char) -> Result<Object,String> {
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


fn get_user_input() -> String {
	let mut line = String::new();
	let stdin = io::stdin();
	return loop {
		stdin.lock().read_line(&mut line).unwrap();
		if line.len() > 0 { break line; }
	}
}


fn new_game(base_level: &Level) -> State {
	// restarts the game, using what's in base_level
	State {
		num_moves: 0,
		move_history: Vec::<Move>::new(),
		human_pos: base_level.human_pos.clone(),
		level: Level {
			w: base_level.w,
			h: base_level.h,
			human_pos: base_level.human_pos.clone(),
			data: base_level.data.clone(),
		}
	}
}


fn _undo_move(base_level: Level, our_state: &mut State)  {
	// resets to a certain spot using move history
	our_state.num_moves = 0;
	our_state.move_history = Vec::<Move>::new();
	our_state.level = base_level;
}


fn _apply_moves(state: &mut State, moves: &Vec::<Move>) -> Result<(),String> {
	let _s = state;
	let _m = moves;
	return Err("Not implemented".to_string());
}


fn main() -> Result<(),String> {
	let mut filename: String = String::from("levels/level01.txt");
	
	let mut count = 0;
	for env in std::env::args() {
		if count > 0 {
			filename = env;
		}
		count += 1;
	}
	
	// load level
	let base_level = load_level(&filename)?;
	
	let mut state = new_game(&base_level);
	
	loop {
		&state.display();
		
		if state.have_win_condition() {
			println!("WIN!");
			break;
		}
		
		print!("Options: qr ");
		let opts = &state.get_move_options();
		for x in opts {
			print!("{}", x.to_string());
		}
		println!(" > ");
		
		let c = get_user_input()[0..1].parse::<char>().unwrap();
	
		match c {
			'q' => break,
			'r' => state = new_game(&base_level),
			'n' => state.apply_move(&Move::North),
			'e' => state.apply_move(&Move::East),
			's' => state.apply_move(&Move::South),
			'w' => state.apply_move(&Move::West),
			_ => {}
		}
	}
	
	return Ok(());
}

