// Box Bopper Tool: Sokoban clone level creator and solver

use std::fs::File;
use std::io::prelude::*;

use boxbopperbase::{Obj};
use boxbopperbase::level::{Level,verify_builtins};
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
	level.make_win_data();

	(level, params)
}

#[derive(Clone)]
struct SpeedTestData {
	pub num: u16,
	pub title: String,
	pub depth: u16,
	pub moves: u16,
	pub path: String,
	pub time: f64,
}

#[derive(Clone)]
struct SpeedTest {
	pub data: Vec::<SpeedTestData>,
}

impl SpeedTest {
	pub fn get_speed_test(&self, num: usize) -> Option::<&SpeedTestData> {
		return self.data.iter().find(|pd| pd.num == num as u16);
	}
	pub fn new() -> SpeedTest {
		SpeedTest {
			data: Vec::<SpeedTestData>::new(),
		}
	}
	pub fn from_file(filename: &str) -> SpeedTest {
		let input = std::fs::read_to_string(filename);
		let input = match input {
			Ok(x) => x,
			_ => panic!("Failed to open Speed Test file: {}", filename),
		};
			
		let p = SpeedTest::from_str(&input);
		p
	}
	pub fn from_str(s: &str) -> SpeedTest {				
		let mut p = SpeedTest::new();
		// load into string
		
		for txt in s.lines() {
			let txt = txt.trim();
			if txt.len() > 0 && txt.chars().nth(0) == Some('#') {
				continue;
			}
			if txt.len() >= 2 {
				let data: Vec<&str> = txt.split(", ").collect();
				if data.len() < 6 {
					println!("Warning: Insufficient columns in read Speed Test row");
					continue;
				}
				let d = SpeedTestData {
					num: data[0].parse::<u16>().expect("Speed test read failed first column"),
					title: data[1].to_string(),
					depth: data[2].parse::<u16>().expect("Speed test read failed third column"),
					moves: data[3].parse::<u16>().expect("Speed test read failed fourth column"),
					path: data[4].to_string(),
					time: data[5].parse::<f64>().expect("Speed test read failed sixth column"),
				};
				p.data.push(d);
			}
		}
		p
	}
}


fn main() -> std::io::Result<()> {
	let args: Vec::<String> = std::env::args().collect();
	#[derive(PartialEq)]
	enum Mode { Help, Solve, Make, SpeedTest };
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
	let mut speed_test_read: String = String::new();
	let mut speed_test_write: String = String::new();
	
	// process params
	for (count,arg) in args.into_iter().enumerate() {
		if count == 1 {
			match arg.as_str() {
				"solve" => { mode = Mode::Solve; },
				"make"  => { mode = Mode::Make; },
				"speed_test" => { mode = Mode::SpeedTest; verbosity = 0; }
				_ => {
					println!("First argument should be make or solve or speed_test");
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
				"speed_test_read"  => { speed_test_read = String::from(right); },
				"speed_test_write"  => { speed_test_write = String::from(right); },
				"builtin"   => { builtin = right.parse::<u32>().unwrap(); }
				"verbosity" => { verbosity = right.parse::<u32>().unwrap(); },
				_ => {
					println!("Unrecognised variable {}", left);
					mode = Mode::Help;
				}
			}
		}
	}

	if width > 127 || height > 127 || width * height > 256 {
		println!("ERROR: Maximum width is 127. Maximum height is 127. Maximum width * height is 256.");
		return Ok(());
	} 

	let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(0x0d47d47000000000_u64 + seed as u64);

	if mode == Mode::Help {
		println!("boxboppertool make [vars...]\nboxboppertool solve [vars...]\nboxboppertool speed_test [vars...]\n");
		println!("vars for make:");
		println!("  seed=n           rng seed (u32)");
		println!("  width=n          level width 5-15                     default: {}", DEF_WIDTH);
		println!("  height=n         level height 5-15                    default: {}", DEF_HEIGHT);
		println!("  box_density=n    box density 1-99                     default: {}", DEF_BOX_DENSITY);
		println!("  wall_density=n   wall density 1-99                    default: {}", DEF_WALL_DENSITY);
		println!("  max_depth=n      maximum depth to try to reach 1+     default: {}", DEF_MAX_DEPTH);
		println!("vars for solve:");
		println!("  max_moves=n      maximum number of moves to try 1+    default: {}", DEF_MAX_MOVES);
		println!("  builtin=n        builtin level to solve");
		println!("  filename=f       custom level filename to solve");
		println!("vars for speed_test:");
		println!("  speed_test_read=f   filename to compare results with");
		println!("  speed_test_write=f  filename to write results to");
		println!("vars for all:");
		println!("  verbosity=n      how much information to provide 0-2  default: {}", DEF_VERBOSITY);
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
		output_str += &format!("time: {:.2}\n", solution.secs);
		output_str += &format!("seed: {}\n", seed);
		output_str += &format!("{}\n", level_params);
		println!("{}",output_str);

		// save level to disk if it meets threshold
		if solution.moves as usize > width*height {
			let filename = format!("levels/rl-{}x{}-b{}-d{}-m{}-t{:.1}-{}.txt", unsolved_level.w, unsolved_level.h, unsolved_level.get_box_count(), solution.depth, solution.moves, solution.secs, unsolved_level.get_title_str());
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

		if width > 127 || height > 127 || width * height > 256 {
			println!("ERROR: Maximum width is 127. Maximum height is 127. Maximum width * height is 256.");
			return Ok(());
		} 

		if filename.len() > 0 {
			println!("Solving level \"{}\" (filename {})...",level.get_title_str(), filename);
		} else {
			println!("Solving level \"{}\" (builtin level {})...",level.get_title_str(), builtin);
		}
		
		if verbosity > 0 { println!("{}",level.to_string()); }

		let solution = solve_level(&level, max_moves, max_maps, verbosity);
		match solution {
			Some(sol) => {
				let mut output_str = "".to_string();
				output_str += &format!("title: {}\n", level.get_title_str());
				output_str += &format!("depth: {}\n", sol.depth);
				output_str += &format!("moves: {}\n", sol.moves);
				output_str += &format!("path: {}\n", sol.path);
				output_str += &format!("time: {:.2}\n", (sol.secs));
				println!("{}", output_str);
			},
			None => {
			},
		};
	} else { // mode = speed_test
		// Solve levels 0 to X and check they solved correctly
		let mut success = true;
		let mut warnings = false;
		let mut solutions = Vec::<Option::<Solution>>::new();
		let mut stored_times = Vec::<f64>::new();
		if !verify_builtins() {
			return Ok(());
		}
		let mut p = SpeedTest::new();
		if speed_test_read.len() > 0 {
			p = SpeedTest::from_file(&speed_test_read);
		}
		let mut save_speed_test_string = String::from("# boxboppertool speed_test\n# num(u16), title(str), depth(u16), moves(u16), path(str), time(f64:s)\n");
		for level_num in 0..=builtin as usize {
			let level = Level::from_builtin(level_num).expect(&format!("Unable to open builtin level {}!", level_num));
			println!("Solving level {} \"{}\"...",level_num,level.get_keyval_or("title","untitled"));
			
			let solution = solve_level(&level, max_moves, max_maps, verbosity);

			match &solution {
				Some(sol) => {
					if speed_test_read.len() > 0 {
						if level_num >= p.data.len() {
							println!("  Level not in read speed_test file");
							success = false;
						} else {
							if p.get_speed_test(level_num).unwrap().title != level.get_keyval_or("title","untitled") {
								println!("  Title mismatch (prev: {}, this: {})", p.get_speed_test(level_num).unwrap().title, level.get_keyval_or("title","untitled"));
								warnings = true;
							}
							if p.get_speed_test(level_num).unwrap().depth != sol.depth {
								println!("  Depth mismatch (prev: {}, this: {})", p.get_speed_test(level_num).unwrap().depth, sol.depth);
								warnings = true;
							}
							if p.get_speed_test(level_num).unwrap().moves != sol.moves {
								println!("  Moves mismatch (prev: {}, this: {})", p.get_speed_test(level_num).unwrap().moves, sol.moves);
								warnings = true;
							}
							if p.get_speed_test(level_num).unwrap().path != sol.path {
								println!("  Path differs\n  prev: {}\n  this: {}", p.get_speed_test(level_num).unwrap().path, sol.path);
								warnings = true;
							}
						}
					}
					save_speed_test_string += &format!("{}, {}, {}, {}, {}, {}\n", level_num, level.get_keyval_or("title","untitled"), sol.depth, sol.moves, sol.path, sol.secs);
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
			if !warnings { println!("Speed Test OK."); }
			else { println!("Speed Test OK (with warnings)."); }
			println!();
			println!("-------------------------------");
			println!(" Level | Prev Time | This Time ");
			println!("-------------------------------");
			let mut total_time = 0_f64;
			let mut total_time_s = 0_f64;
			for level_num in 0..=builtin as usize {
				let t = solutions[level_num].as_ref().unwrap().secs;
				total_time += t;
				if speed_test_read.len() > 0 {
					let pld = p.get_speed_test(level_num);
					let pt = if pld.is_some() { pld.unwrap().time } else { 0_f64 };
					total_time_s += pt;
					println!("  {:>4}   {:>9.3}   {:>9.3}", level_num, pt, t);
				} else  {
					println!("  {:>4}   {:>9}   {:>9.3}", level_num, " ", t);
				}
			}
			println!("-------------------------------");
			if speed_test_read.len() > 0 {
				println!(" Total | {:>9.3} | {:>9.3}", total_time_s, total_time);
				println!("-------------------------------");
				println!(" Improvement: {:>4.1}%", (total_time_s - total_time) / total_time_s * 100_f64);
				} else {
				println!(" Total | {:>9} | {:>9.3}", " ", total_time);
				println!("-------------------------------");
			}
			if speed_test_write.len() > 0 {
				let result = std::fs::write(&speed_test_write, save_speed_test_string);
				if result.is_ok() {
					println!("Speed Test saved to file {}", speed_test_write);
				} else {
					println!("Failed to save Speed Test to file {}", speed_test_write);
				}
			}
		} else {
			println!();
			println!("Speed Test failed.");
			println!();
		}
	}

	return Ok(());
}

