
// Box Bopper: Sokoban clone in rust

//use std::io;
//use std::io::{BufReader,BufRead};
//use std::fs::File;

//mod boxbopperbase;

use boxbopperbase::{Game};
use boxbopperbase::level::{load_level};

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
	
	let state = Game::new_from_level(&base_level,0);
	
	loop {
		// TODO

		let opts = &state.get_move_options();
		for x in opts {
			print!("{}", x.to_string());
		}
	
		break;
	}
	
	return Ok(());
}

