
// Box Bopper: Sokoban clone in rust

use std::io;
use std::io::BufRead;
//use std::fs::File;

mod boxbopperbase;
use ::boxbopperbase::{Game};

mod vector;
use vector::{Vector,Move,ALLMOVES};

mod level;
use level::{Level,load_builtin};

mod builtins;
use builtins::BUILTIN_LEVELS;

pub fn get_user_input() -> String {
	let mut line = String::new();
	let stdin = io::stdin();
	return loop {
		stdin.lock().read_line(&mut line).unwrap();
		if line.len() > 0 { break line; }
	}
}


fn main() -> Result<(),String> {
	let mut _filename: String = String::from("levels/level01.txt");
	
	let mut count = 0;
	for env in std::env::args() {
		if count > 0 {
			_filename = env;
		}
		count += 1;
	}
	
	// load level
	//let base_level = load_level(&filename)?;
	let mut current_level: u32 = 0;
	
	let mut state = Game::new(0);
	
	loop {
		&state.display();
		
		if state.level.have_win_condition() {
			println!(r"\  /\  / | |\ |");
			println!(r" \/  \/  | | \|");
			if current_level < BUILTIN_LEVELS.len() as u32 {
				current_level += 1;
			} else {			
				println!("All levels complete!");
				break;
			};
			state = Game::new(current_level);
			println!("Level {}", current_level);
			&state.display();
		}
		
		print!("Options: q` ");
		let opts = &state.get_move_options();
		for x in opts {
			print!("{}", x.to_string());
		}
		println!(" > ");
		
		let c = get_user_input()[0..1].parse::<char>().unwrap();
	
		match c {
			'q' | 'Q' => break,
			'`' => state = Game::new(current_level),
			'n' =>  { if current_level < BUILTIN_LEVELS.len() as u32 {
						current_level += 1;
						state = Game::new(current_level);
						println!("Level {}", current_level);
					}},
			'p' => { if current_level > 0 {
						current_level -= 1;
						state = Game::new(current_level);
						println!("Level {}", current_level);
					}},
			'u' | 'U' => state.apply_move(&Move::Up),
			'r' | 'R' => state.apply_move(&Move::Right),
			'd' | 'D' => state.apply_move(&Move::Down),
			'l' | 'L' => state.apply_move(&Move::Left),
			_ => {}
		}
	}
	
	return Ok(());
}

