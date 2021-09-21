// Box Bopper: Sokoban clone in rust
// Copyright David Atkinson 2020-2021
//
// boxbopperconsole.rs: console game player

use std::io;
use std::io::BufRead;

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

const DEF_VERBOSITY: u32 = 1;

fn main() -> Result<(),String> {
	let args: Vec::<String> = std::env::args().collect();
	let mut filename: String = String::from("");
	let mut builtin: u32 = 0;
	let mut verbosity: u32 = DEF_VERBOSITY;
	
	// process params
	for (count,arg) in args.into_iter().enumerate() {
		if count >= 1 {
			let eq_idx = arg.find('=');
			if eq_idx.is_none() {
				println!("No equals symbol found in var");
			}
			let eq_idx = eq_idx.unwrap();
			let left = &arg[0..eq_idx];
			let right = &arg[eq_idx+1..];
			match left {
				"filename"  => { filename = String::from(right); },
				"builtin"   => { builtin = right.parse::<u32>().unwrap(); }
				"verbosity" => { verbosity = right.parse::<u32>().unwrap(); },
				_ => {
					println!("Unrecognised variable {}", left);
				}
			}
		}
	}

	
	let mut state = if filename.len() > 0 {
		Game::new_from_level(&Level::from_file(&filename).expect("Unable to open specified file"), 0)
	} else {
		Game::new(builtin)
	};
		
	let mut current_level: u32 = builtin;
	let mut quit = false;
	
	while !quit {
		while state.is_queued_moves() {
			state.process_moves();
		}
		
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
		
		//let c = get_user_input()[0..1].parse::<char>().unwrap();
		
		get_user_input().chars().for_each( |c| match c {
			'q' | 'Q' => quit = true,
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
		});
	}
	
	return Ok(());
}

