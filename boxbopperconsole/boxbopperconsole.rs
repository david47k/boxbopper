
// Box Bopper: Sokoban clone in rust

use std::io;
use std::io::BufRead;
//use std::fs::File;

use boxbopperbase::{Game};
use boxbopperbase::vector::{Move};
//use boxbopperbase::level::{Level,load_builtin};
use boxbopperbase::builtins::BUILTIN_LEVELS;

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
		state.process_moves();
		
		&state.display();
		
		if state.have_win_condition() {
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
			'u' | 'U' => state.append_move(&Move::Up),
			'r' | 'R' => state.append_move(&Move::Right),
			'd' | 'D' => state.append_move(&Move::Down),
			'l' | 'L' => state.append_move(&Move::Left),
			_ => {}
		}
	}
	
	return Ok(());
}

