// Box Bopper Tool: Sokoban clone level creator and solver

use std::fs::File;
use std::io::prelude::*;

use boxbopperbase::{Obj};
use boxbopperbase::level::{Level};
use boxbopperbase::vector::{Vector,ALLMOVES};

pub mod defs;
use defs::{*};

pub mod solve;
use solve::{solve_level,Solution};

pub mod unsolve;
use unsolve::{unsolve_level};

pub mod pathnodemap;

extern crate rand;
extern crate rand_chacha;

use rand::{Rng, SeedableRng};


fn is_pullable(level: &Level, pos: &Vector) -> bool {
	// check all four directions, and see if we can pull in that direction
	// i.e. one of the four directions must have non-wall,non-wall. 
	let mut ok = false;
	for m in ALLMOVES.iter() {
		let p1 = pos.add(&m.to_vector());
		let p2 = pos.add(&m.to_vector().double());
		if !level.vector_in_bounds(&p1) || !level.vector_in_bounds(&p2) { continue; }

		let o1 = level.get_obj_at_pt(&p1);
		let o2 = level.get_obj_at_pt(&p2);
		if o1 != Obj::Wall && o2 != Obj::Wall {
			ok = true;
		}
	}

	ok
}


fn random_string(rng: &mut rand_chacha::ChaCha8Rng) -> String {
	let k = ['b','d','f','g','h','j','k','l','m','n','p','r','s','t','v','w','y','z']; //18
	let v = ['a','e','i','o','u']; //5
	// 18*5 = 90, 90^5 > 2^32

	let mut s = String::new();
	for _ in 0..5 {
		s += &k[rng.gen_range(0,k.len())].to_string();
		s += &v[rng.gen_range(0,v.len())].to_string();
	}
	s
}


fn random_level_creator(width: u16, height: u16, wall_density: u32, box_density: u32, rng: &mut rand_chacha::ChaCha8Rng) -> (Level,String) {
	let mut data = Vec::<Obj>::with_capacity(width as usize * height as usize);
	let mut params = String::new();

	params += &format!("width: {}\nheight: {}\n", width, height);

	// fill with spaces
	for _n in 0..(width * height) {
		data.push(Obj::Space);
	}

	// randomly place us
	let x = rng.gen_range(0,width);
	let y = rng.gen_range(0,height);
	data[(y*width + x) as usize] = Obj::Human;
	let human_pos = Vector(x as i32,y as i32);
	
	// randomly place walls - not on anything else
	for y in 0..height as usize {
		for x in 0..width as usize {
			if data[y*width as usize+x] == Obj::Space && rng.gen_range(0,100) <= wall_density {
				data[y*width as usize+x] = Obj::Wall;
			}
		}
	}
	params += &format!("wall_density: {}\n", wall_density);

	// create the level
	let mut level = Level::from_parts(random_string(rng), width, height, human_pos, data);
	//let mut boxx_pts = Vec::<Vector>::new();
	let mut hole_pts = Vec::<Vector>::new();

	// calculate how many boxxes
	let max_squares = width as usize * height as usize;
	let mut num_boxxes = max_squares * box_density as usize / 100;
	if num_boxxes < 3 { num_boxxes = 3; };
	
	// place the boxxes
	let mut i = 0;
	let mut insane = 0;
	while i < num_boxxes && insane < max_squares * 10 { // don't let it run forever
		let x = rng.gen_range(0,width);
		let y = rng.gen_range(0,height);
		let v = Vector(x as i32, y as i32);
		if level.get_obj_at_pt(&v) == Obj::Space {
			if is_pullable(&level, &v) {
				level.set_obj_at_pt(&v, Obj::BoxxInHole);
				hole_pts.push(v);
				i+=1;
			}
		}
		insane += 1;
	}
	if i != num_boxxes {
		println!("Warning: unable to place {} boxes, only placed {} boxes.", num_boxxes, i);
		num_boxxes = i;
	}
	params += &format!("box_density: {}\n", box_density);
	params += &format!("num_boxxes: {}\n", num_boxxes);

	level.do_noboxx_pts();
	level.do_boxx_pts();

	(level, params)
}


fn main() -> std::io::Result<()> {
	let args: Vec::<String> = std::env::args().collect();
	#[derive(PartialEq)]
	enum Mode { Help, Solve, Make, Profile };
	let mut mode = Mode::Help;
	let mut seed: u32 = 0;
	let mut max_moves: u16 = DEF_MAX_MOVES;
	let mut max_depth: u16 = DEF_MAX_DEPTH;
	let mut width: usize = DEF_WIDTH;
	let mut height: usize = DEF_HEIGHT;
	let mut box_density: u32 = DEF_BOX_DENSITY;
	let mut wall_density: u32 = DEF_WALL_DENSITY;
	let max_maps: usize = DEF_MAX_MAPS;
	let mut filename: String = String::from("");
	let mut builtin: u32 = 0;
	let mut verbosity: u32 = DEF_VERBOSITY;
	
	// process params
	for (count,arg) in args.into_iter().enumerate() {
		if count == 1 {
			match arg.as_str() {
				"solve" => { mode = Mode::Solve; },
				"make"  => { mode = Mode::Make; },
				"profile" => { mode = Mode::Profile; verbosity = 0; }
				_ => {
					println!("First argument should be make or solve or profile");
				}
			};
		} else if count >= 2 {
			let eq_idx = arg.find('=');
			if eq_idx.is_none() {
				println!("No equals symbol found in var");
				mode = Mode::Help;
			}
			let eq_idx = eq_idx.unwrap();
			let left = &arg[0..eq_idx];
			let right = &arg[eq_idx+1..];
			match left {
				"seed" => { seed = right.parse::<u32>().unwrap(); },
				"width" => { width = right.parse::<usize>().unwrap(); },
				"height" => { height = right.parse::<usize>().unwrap(); },
				"box_density" => { box_density = right.parse::<u32>().unwrap(); },
				"wall_density" => { wall_density = right.parse::<u32>().unwrap(); },
				"max_moves" => { max_moves = right.parse::<u16>().unwrap(); },
				"max_depth" => { max_depth = right.parse::<u16>().unwrap(); },
				"filename"  => { filename = String::from(right); },
				"builtin"   => { builtin = right.parse::<u32>().unwrap(); }
				"verbosity" => { verbosity = right.parse::<u32>().unwrap(); },
				_ => {
					println!("Unrecognised variable {}", left);
					mode = Mode::Help;
				}
			}
		}
	}

	if width > 127 || height > 127 || width * height > 240 {
		println!("ERROR: Maximum width is 127. Maximum height is 127. Maximum width * height is 240.");
		return Ok(());
	} 

	let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(0x0d47d47000000000_u64 + seed as u64);

	if mode == Mode::Help {
		println!("boxboppertool make [vars...]\nboxboppertool solve [vars...]\nboxboppertool profile [vars...]\n");
		println!("vars for make:");
		println!("  seed=n          rng seed (u32)");
		println!("  width=n         level width 5-15                    default: {}", DEF_WIDTH);
		println!("  height=n        level height 5-15                   default: {}", DEF_HEIGHT);
		println!("  box_density=n   box density 1-99                    default: {}", DEF_BOX_DENSITY);
		println!("  wall_density=n  wall density 1-99                   default: {}", DEF_WALL_DENSITY);
		println!("  max_depth=n     maximum depth to try to reach 1+    default: {}", DEF_MAX_DEPTH);
		println!("vars for both:");
		println!("  verbosity=n     how much information to provide 0-2 default: {}", DEF_VERBOSITY);
		println!("vars for solve:");
		println!("  max_moves=n     maximum number of moves to try 1+   default: {}", DEF_MAX_MOVES);
		println!("  builtin=n       builtin level to solve");
		println!("  filename=f      custom level filename to solve");
		println!("");
	} else if mode == Mode::Make {
		// create level
		let (random_level, level_params) = random_level_creator(width as u16, height as u16, wall_density, box_density, &mut rng);

		// unsolve the level
		if verbosity > 0 { 
			println!("==== Unsolving level ===="); 
			println!("{}", &random_level.to_string());
		}
		let unsolved_levels = unsolve_level(&random_level, max_depth, max_maps, &mut rng, verbosity);

		let mut best_idx = None;
		let mut solutions = Vec::<Option<Solution>>::new();
		for x in 0..unsolved_levels.len() {
			println!("==== Solving variation {} of {} ====", x, unsolved_levels.len()-1);
			println!("{}", &unsolved_levels[x].to_string());
			let solution = solve_level(&unsolved_levels[x], unsolved_levels[x].get_keyval("moves").parse::<u16>().expect("number->string->number failure!")+2, max_maps, verbosity); // probably don't need the +2
			solutions.push(solution.clone());
			match solution {
				Some(solution) => {
					if best_idx.is_none() {
						best_idx = Some(x);
					}
					if solutions.len() > 0 {
						if solution.depth >= solutions[best_idx.unwrap()].as_ref().unwrap().depth && solution.moves >= solutions[best_idx.unwrap()].as_ref().unwrap().moves {
							best_idx = Some(x);
						}
					}
				},
				None => {
					// No solutions found
				}
			}
		}

		if best_idx.is_none() {
			println!("==== No solutions found ====");
			return Ok(());
		}

		println!("==== Solutions found ====");

		// display results
		for (i,s) in solutions.iter().enumerate() {			
			let sol_depth;
			let sol_moves;
			if s.is_some() {
				sol_depth = s.clone().unwrap().depth.to_string();
				sol_moves = s.clone().unwrap().moves.to_string();
			} else {
				sol_depth = "unsolved".to_string();
				sol_moves = "unsolved".to_string();
			}
			if unsolved_levels[i].get_keyval("depth") == sol_depth && unsolved_levels[i].get_keyval("moves") == sol_moves {
				println!("Variation {}: depth {}, moves {}", i, sol_depth, sol_moves);
			} else {
				println!("Variation {}: depth {} -> {}, moves {} -> {}",i,unsolved_levels[i].get_keyval("depth"),sol_depth,unsolved_levels[i].get_keyval("moves"),sol_moves);
			}
		}

		// pick best level
		let solution = solutions[best_idx.unwrap()].as_ref().unwrap();
		let mut unsolved_level = unsolved_levels[best_idx.unwrap()].clone();
		unsolved_level.set_keyval("title",&format!("{}-{}",unsolved_level.get_title_str(),solution.moves));

		println!("-- Chosen level {} --", best_idx.unwrap());
		let mut output_str = String::new();
		output_str += &format!("{}\n",unsolved_level.to_string());
		output_str += &format!("title: {}\n", unsolved_level.get_title_str());
		output_str += &format!("depth: {}\n", solution.depth);
		output_str += &format!("moves: {}\n", solution.moves);
		output_str += &format!("path: {}\n", solution.path);
		output_str += &format!("time: {:.1}\n", (solution.msecs+500_f64)/1000_f64);
		output_str += &format!("seed: {}\n", seed);
		output_str += &format!("{}\n", level_params);
		println!("{}",output_str);

		// save level to disk if it meets threshold
		if solution.moves as usize > (width-1)*(height-1) {
			let filename = format!("levels/rl-{}x{}-b{}-d{}-m{}-t{:.1}-{}.txt", unsolved_level.w, unsolved_level.h, unsolved_level.get_box_count(), solution.depth, solution.moves, solution.msecs/1000_f64, unsolved_level.get_title_str());
			let mut fout = File::create(&filename).unwrap();
			let result = fout.write_all(output_str.as_bytes());
			if !result.is_ok() {
				println!("Failed to save level to filename: {}", filename);
				return Err(std::io::Error::last_os_error());
			}
		}
	} else if mode == Mode::Solve {
		// load level
		let level = if filename.len() > 0 {
			Level::from_file(&filename).expect("Unable to open specified file")
		} else {
			Level::from_builtin(builtin as usize).expect(&format!("Unable to open builtin level {}!", builtin))
		};
		
		println!("{}",level.to_string());

		if width > 127 || height > 127 || width * height > 240 {
			println!("ERROR: Maximum width is 127. Maximum height is 127. Maximum width * height is 240.");
			return Ok(());
		} 

		let solution = solve_level(&level, max_moves, max_maps, verbosity);
		match solution {
			Some(sol) => {
				let mut output_str = "".to_string();
				output_str += &format!("title: {}\n", level.get_title_str());
				output_str += &format!("depth: {}\n", sol.depth);
				output_str += &format!("moves: {}\n", sol.moves);
				output_str += &format!("path: {}\n", sol.path);
				output_str += &format!("time: {:.1}\n", (sol.msecs/1000_f64));
				println!("{}", output_str);
			},
			None => {
			},
		};
	} else { // mode = profile
		// Solve levels 1-20 and check they solved correctly
		let mut success = true;
		let mut solutions = Vec::<Option::<Solution>>::new();
		let mut stored_times = Vec::<f64>::new();
		for level_num in 0..20 {
			let level = Level::from_builtin(level_num).expect(&format!("Unable to open builtin level {}!", builtin));
			println!("Solving level {}...",level_num);

			let solution = solve_level(&level, max_moves, max_maps, verbosity);
			match &solution {
				Some(sol) => {
					if level.get_keyval("depth").parse::<u16>().expect("Builtin depth error") != sol.depth {
						println!("  Depth mismatch (stored: {}, profiled: {})", level.get_keyval("depth"), sol.depth);
						success = false;
					}
					if level.get_keyval("moves").parse::<u16>().expect("Builtin moves error") != sol.moves {
						println!("  Moves mismatch (stored: {}, profiled: {})", level.get_keyval("moves"), sol.moves);
						success = false;
					}
					if level.get_keyval("path") != sol.path {
						println!("  Path differs\n  Stored:   {}\n  Profiled: {}", level.get_keyval("path"), sol.path);
					}
					stored_times.push(level.get_keyval("time").parse::<f64>().expect("Builtin level time error"));
				},
				None => {
					println!("  Failed to find solution");
					stored_times.push(0_f64);
					success = false;
				}
			}
			solutions.push(solution);
		}
		if success {
			println!();
			println!("Profile OK.");
			println!();
			println!("-------------------------------------");
			println!(" Level | Stored Time | Profiled Time ");
			println!("-------------------------------------");
			let mut total_time_s = 0_f64;
			let mut total_time = 0_f64;
			for level_num in 0..20 {
				total_time_s += stored_times[level_num];
				let t = solutions[level_num].as_ref().unwrap().msecs / 1000_f64;
				total_time += t;
				println!("   {:>2}    {:>8.3}      {:>8.3}", level_num, stored_times[level_num], t);
			}
			println!("-------------------------------------");
			println!(" Total | {:>8.3}    | {:>8.3}", total_time_s, total_time);
			println!("-------------------------------------");
			println!(" Improvement: {:>4.1}%", (total_time_s - total_time) / total_time_s * 100_f64);
		} else {
			println!();
			println!("Profile failed.");
			println!();
		}
	}

	return Ok(());
}

