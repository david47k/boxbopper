
// Box Bopper: Sokoban clone in rust

use std::io;
use std::io::BufRead;
//use std::fs::File;

use boxbopperbase::{Game};
use boxbopperbase::vector::{Move};
use boxbopperbase::level::{Level};
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
	let mut filename: String = String::from("");
	
	let mut count = 0;
	for env in std::env::args() {
		if count > 0 {
			filename = env;
		}
		count += 1;
	}
	
	let custom_level;
	let mut state;
	
	// load level
	if filename.len() > 0 {
		custom_level = Level::from_file(&filename).expect("Unable to open custom level file");
		state = Game::new_from_level(&custom_level,0);
	} else {
		state = Game::new(0);
	}
	
	let mut current_level: u32 = 0;	
	
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

