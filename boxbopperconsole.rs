
// Box Bopper: Sokoban clone in rust

use std::io;
use std::io::BufRead;
//use std::fs::File;

//mod boxbopperbase;

use boxbopperbase::{Move,Game,load_builtin};


pub fn get_user_input() -> String {
	let mut line = String::new();
	let stdin = io::stdin();
	return loop {
		stdin.lock().read_line(&mut line).unwrap();
		if line.len() > 0 { break line; }
	}
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
	//let base_level = load_level(&filename)?;
	let mut current_level: usize = 0;
	let mut base_level = load_builtin(current_level).unwrap();
	
	let mut state = Game::new(&base_level);
	
	loop {
		&state.display();
		
		if state.have_win_condition() {
			println!(r"\  /\  / | |\ |");
			println!(r" \/  \/  | | \|");
			current_level += 1;
			base_level = match load_builtin(current_level) {
				Some(lvl) => lvl,
				None => {
					println!("All levels complete!");
					break;
				},
			};
			state = Game::new(&base_level);
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
			'`' => state = Game::new(&base_level),
			'u' | 'U' => state.apply_move(&Move::Up),
			'r' | 'R' => state.apply_move(&Move::Right),
			'd' | 'D' => state.apply_move(&Move::Down),
			'l' | 'L' => state.apply_move(&Move::Left),
			_ => {}
		}
	}
	
	return Ok(());
}

