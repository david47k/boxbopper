// Box Bopper Tool: Sokoban clone level creator and solver

use std::cmp::Ordering;
use rayon::prelude::*;
use std::fs::File;
use std::io::prelude::*;
use std::sync::{Arc,Mutex};
use std::sync::atomic::Ordering as AtomicOrdering;
use std::sync::atomic::*;
use std::rc::Rc;

use boxbopperbase::{Obj,moves_to_string};
use boxbopperbase::level::{Level};
use boxbopperbase::vector::{Vector,ALLMOVES,Move,ShrunkPath};
use boxbopperbase::time::{get_time_ms};

pub mod pathnodemap;
use crate::pathnodemap::{PathNodeMap,KeyMove};



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


#[derive(Clone)]
pub struct Solution {
	pub moves: u16,
	pub depth: u16,
	pub msecs: f64,
	pub path: String
}


pub fn solve_level(base_level: &Level, max_moves_requested: u16, max_maps: usize, rng: &mut rand_chacha::ChaCha8Rng, verbosity: u32) -> Option<Solution> {
	let max_moves = Arc::new(AtomicU16::new(max_moves_requested+1));
	let mut base_level = base_level.clone();
	base_level.clear_human();
	let base_level = &base_level;
	let base_map = PathNodeMap::new_from_level(base_level);

	let mut non_contenders = Vec::<u128>::with_capacity(50000);
	let mut mapsr = Rc::new(vec![base_map]);
	
	let have_solution = Arc::new(AtomicBool::new(false));
	let best_solution_str = Arc::new(Mutex::new(String::new()));
	let best_sol_depth = Arc::new(AtomicU16::new(0));
	let mut depth: u16 = 0;

	let msecs0 = get_time_ms();

	while depth < max_moves.load(AtomicOrdering::SeqCst) {
		if verbosity > 0 { println!("-- Depth {:>2} --", depth); }

		// check for level complete / having solution
		if verbosity > 1 { println!("solution check..."); }
		mapsr.par_iter().filter(|m| m.is_level_complete(base_level)).for_each(|m| {
			if m.nodes[0].steps < max_moves.load(AtomicOrdering::SeqCst) {
				have_solution.store(true, AtomicOrdering::SeqCst);
				max_moves.store(m.nodes[0].steps, AtomicOrdering::SeqCst);
				best_sol_depth.store(depth, AtomicOrdering::SeqCst);
				let mut solstr = best_solution_str.lock().unwrap();
				*solstr = format!("{}", &m.path.to_string());
				if verbosity > 0 { 
					println!("-- Solution found in {} moves --", m.nodes[0].steps);
				}
			}
		});

		// complete the maps
		if verbosity > 1 { println!("completing  {:>7} maps", mapsr.len()); }
		let maps: Vec<PathNodeMap> = mapsr.par_iter().map(|m| m.complete_map_solve(base_level) ).collect(); // collect_into_vec doesn't seem to be any faster

		// apply key moves
		if verbosity > 1 { println!("collecting kms..."); }
		let todo_list: Vec<(&PathNodeMap,&KeyMove)> = maps.iter().flat_map(|m| m.key_moves.iter().map(|mv| (m,mv)).collect::<Vec::<(&PathNodeMap,&KeyMove)>>() ).collect();
		if verbosity > 1 { println!("applying kms..."); }
		let mut maps: Vec<PathNodeMap> = todo_list.par_iter().map(|(m,mv)| m.new_by_applying_key_push(mv)).collect();

		// filter out the long paths
		if verbosity > 1 { println!("pruning long paths..."); }
		let ms = max_moves.load(AtomicOrdering::SeqCst);
		maps.retain(|m| m.nodes[0].steps < ms);

		// sort and deduplicate
		if depth >= 2 { 
			if verbosity > 1 { println!("deduping: before {:>7}", maps.len()); }
			dedupe_equal_levels(&mut maps);
			if verbosity > 1 { println!("deduping: after  {:>7}", maps.len()); }
		} 

		// TO DO: Shrink mapsr, add it to non-contenders, sort, dedupe current maps against non-contenders
		if verbosity > 1 { println!("Adding {} old maps to non-contenders...", mapsr.len()); }
		let mut data: Vec::<u128> = mapsr.iter().map(|m| m.level.cmp_data).collect();
		non_contenders.append(&mut data);
		if verbosity > 1 { println!("Sorting {} non_contenders...", non_contenders.len()); }
		non_contenders.par_sort_unstable();

		// remove from maps, anyhthing that is in non_contenders
		if verbosity > 1 { println!("deduping from n-c: before {:>7}", maps.len()); }
		maps.par_iter_mut().for_each(|m| if non_contenders.binary_search(&m.level.cmp_data).is_ok() {
			m.flag = true;
		});
		maps.retain(|m| !m.flag);
		if verbosity > 1 { println!("deduping from n-c: after  {:>7}", maps.len()); }

		// check if we've exhausted the search space
		if maps.len() == 0 {
			if verbosity > 0 { println!("-- No more maps to check --"); }
			break;
		}

		if maps.len() > max_maps {
			println!("--- Hit maximum maps ({}) ---",max_maps);
			while maps.len() > max_maps {
				maps.retain(|_m| rng.gen());
			}
		}
		
		// loop and check the next depth
		mapsr = Rc::new(maps);
		depth += 1;
	}

	if have_solution.load(AtomicOrdering::SeqCst) && base_level.get_box_count()>0 {
		let solstr = best_solution_str.lock().unwrap();		
		if verbosity > 0 { 
			println!("-- Best solution --");
			println!("Solution in {} moves: {}",max_moves.load(AtomicOrdering::SeqCst), solstr);
		}
		return Some(Solution {
			msecs: get_time_ms() - msecs0,
			moves: max_moves.load(AtomicOrdering::SeqCst),
			depth: best_sol_depth.load(AtomicOrdering::SeqCst),
			path: solstr.to_string(),
		});
	} else {
		let ms = max_moves.load(AtomicOrdering::SeqCst);
		if verbosity > 0 {
			println!("-- No solution found --");
			if ms > 1 { println!("Max steps was {}",ms-1); }
		}
		return None;
	}

}


// level equality first, then maximize depth, then minimize moves
pub fn pnm_cmp(a: &PathNodeMap, b: &PathNodeMap) -> Ordering {
	let ord = a.level.partial_cmp(&b.level).unwrap();
	if ord == Ordering::Equal {
		if a.depth > b.depth {
			return Ordering::Less;
		}
		if a.depth < b.depth {
			return Ordering::Greater;
		} 
		if a.nodes[0].steps < b.nodes[0].steps {
			return Ordering::Less;
		}
		if a.nodes[0].steps > b.nodes[0].steps {
			return Ordering::Greater;
		}
	}
	ord
}


// maximise depth, otherwise equal
pub fn pnm_cmp_d(a: &PathNodeMap, b: &PathNodeMap) -> Ordering {
	if a.depth > b.depth {
		return Ordering::Less;
	}
	if a.depth < b.depth {
		return Ordering::Greater;
	}
	return Ordering::Equal;
}


fn dedupe_equal_levels(maps: &mut Vec::<PathNodeMap>) {
	maps.par_sort_unstable_by(|a,b| {
		let ord = a.level.cmp_data.partial_cmp(&b.level.cmp_data).unwrap();
		//let ord = a.level.partial_cmp(&b.level).unwrap();
		if ord == Ordering::Equal {
			if a.nodes[0].steps < b.nodes[0].steps {
				return Ordering::Less;
			}
			if a.nodes[0].steps > b.nodes[0].steps {
				return Ordering::Greater;
			}
		}
		ord			
	});
	maps.dedup_by(|a,b| a.level.cmp_data.eq(&b.level.cmp_data)); // it keeps the first match for each level (sorted to be smallest steps)
}


fn select_unique_n_from(count: usize, len: usize, rng: &mut rand_chacha::ChaCha8Rng) -> Vec::<usize> {
	if len <= count {
		return (0..len).collect();
	} else if len < 100000 {
		let mut range: Vec::<usize> = (0..len).collect();
		range.sort_by_cached_key(|_x| rng.gen_range(0,usize::MAX));
		return Vec::from(&range[0..count]);
	} else {
		// use less ram for big ranges
		let mut selected_idx = Vec::<usize>::with_capacity(count);		
		let mut everloop_count = 0;
		while selected_idx.len() < count && everloop_count < 10000 {
			let idx = rng.gen_range(0,len);
			if !selected_idx.iter().any(|i| *i==idx) {
				selected_idx.push(idx);
			} else {
				everloop_count += 1;
			}
		}
		return selected_idx;
	}
}


pub fn unsolve_level(base_level: &Level, max_depth: u16, max_maps: usize, rng: &mut rand_chacha::ChaCha8Rng, verbosity: u32) -> Vec::<Level> {
	let mut base_level = base_level.clone();
	base_level.clear_human();
	let base_level = &base_level;
	let base_map = PathNodeMap::new_from_level(base_level);

	// A map is complete when the last box is pushed into place. So when unsolving, we need to start with the human
	// in the appropriate spot(s) they'd be after pushing the last box.
	// To do this, we unsolve once to find the appropriate spot(s), then re-solve to place the human and box in the final state.

	if verbosity > 1 { println!("finding final maps..."); }
	let mapsr: Vec<PathNodeMap> = vec![base_map].iter().map(|m| m.complete_map_unsolve(base_level) ).collect();
	let mapsr: Vec<PathNodeMap> = mapsr.iter().flat_map(|map| map.apply_key_pulls(0) ).collect();
	let mapsr: Vec<PathNodeMap> = mapsr.iter().map(|m| m.complete_map_solve(base_level) ).collect();
	let mapsr: Vec<PathNodeMap> = mapsr.iter().flat_map(|map| map.apply_key_pushes() ).collect();
	let mut mapsr: Vec<PathNodeMap> = mapsr.iter().filter(|m| m.is_level_complete(base_level) ).cloned().collect();
	mapsr.iter_mut().for_each(|mut map| { 
		map.path = ShrunkPath::new(); 
		map.nodes[0].steps = 0;
	});
	if verbosity > 1 { 
		println!("final maps found: {}", mapsr.len()); 
		for m in mapsr.iter() {
			println!("{}",m.level.to_string());
		}
	}

	let mut non_contenders = Vec::<u128>::new();
	let mut contenders = Vec::<PathNodeMap>::new();
	mapsr.par_iter_mut().for_each(|m| m.level.make_cmp_data_fast_128() );
	let mut mapsr = Rc::new(mapsr);

	for count in 0..=(max_depth+1) {
		println!("--- Depth {:>2} ---", count);
		
		// complete the maps (finding keymoves as it goes)
		if verbosity > 1 { println!("completing  {:>7} maps", mapsr.len()); }
		let maps: Vec<PathNodeMap> = mapsr.par_iter().map(|m| m.complete_map_unsolve(base_level) ).collect();

		// apply key moves
		if verbosity > 1 { println!("collecting kms..."); }
		let todo_list: Vec<(&PathNodeMap,&KeyMove)> = maps.iter().flat_map(|m| m.key_moves.iter().map(|mv| (m,mv)).collect::<Vec::<(&PathNodeMap,&KeyMove)>>() ).collect();
		if verbosity > 1 { println!("applying kms..."); }
		let mut maps: Vec<PathNodeMap> = todo_list.par_iter().map(|(m,mv)| m.new_by_applying_key_pull(mv,count+1)).collect();
		
		// sort and deduplicate
		if verbosity > 1 { println!("deduping: before {:>7}", maps.len()); }
		dedupe_equal_levels(&mut maps);
		if verbosity > 1 { println!("deduping: after  {:>7}", maps.len()); }

		// mapsr --> contenders --> non-contenders	
		if verbosity > 1 { println!("keep top 20 contenders..."); }

		// save mapsr to contenders
		contenders.append(Rc::get_mut(&mut mapsr).expect("couldn't get mut mapsr"));

		// keep top 20 contenders
		if contenders.len() > 20 {
			// save excess contenders to non-contenders
			let keep = contenders.split_off(contenders.len()-20);
			let mut data = contenders.iter().map(|m| m.level.cmp_data).collect();
			non_contenders.append(&mut data);			// save whats now in contendrs to non-contenders			
			contenders = keep;					// copy keep back
		}

		// sort non-contenders, lets us do binary search
		if verbosity > 1 { println!("sorting non-contenders..."); }
		non_contenders.par_sort_unstable();

		// remove from maps, anyhthing that is in non_contenders
		if verbosity > 1 { println!("deduping from n-c: before {:>7}", maps.len()); }
		maps.par_iter_mut().for_each(|m| if non_contenders.binary_search(&m.level.cmp_data).is_ok() {
			m.flag = true;
		});
		maps.retain(|m| !m.flag);
		if verbosity > 1 { println!("deduping from n-c: after  {:>7}", maps.len()); }
		
		// check if we've run out of options, if we have, then contenders is what we have
		if maps.len() == 0 {
			if verbosity > 0 { println!("-- Out of options (1) (no further moves possible) --"); }
			break;
		}		

		// check if we've hit max depth, in which case we have maps / contenders
		if count > max_depth {
			// check if we've run out of options
			if verbosity > 0 { println!("-- Out of options (3) (hit depth limit) --"); }
			contenders.append(&mut maps);
			break;
		}

		// check if we have waaaaaaaaaaay too many maps
		if maps.len() > max_maps/8 {
			// These are all at same depth so we can just randomly reduce it
			println!("--- Hit maximum unsolve maps {} ---",max_maps/8); 
			while maps.len() > max_maps {
				maps.retain(|_m| rng.gen());
			}
		}

		mapsr = Rc::new(maps);
	}

	if verbosity > 1 { println!("Max depth was {}",max_depth); }
	if contenders.len() == 0 {
		println!("-- No maps to choose from! --");
		return Vec::<Level>::new();
	}

	if verbosity > 0 { print!("Contenders size {} -> ",contenders.len()); }

	// re-sort by depth
	contenders.par_sort_unstable_by(|a,b| pnm_cmp_d(a,b));

	let mut truncsize = 3;
	if contenders.len() >= 80 {
		truncsize = 20;
	} else if contenders.len() >= 12 {
		truncsize = contenders.len() / 4;
	}
	contenders.truncate(truncsize);
	if verbosity > 0 { 
		println!("{}",contenders.len()); 
		print!("(depth,moves): ");
		for c in &contenders {
			print!("({},{}) ", c.depth, c.path.len());
		}
		println!("");
	}
	
	if verbosity > 1 { println!("Picking up to 3 random contenders"); }
	let mut levels = Vec::<Level>::new();
	let selected_idx = select_unique_n_from(3,contenders.len(),rng);
	for idx in selected_idx {
		let c = &contenders[idx];
		let splevel = &c.level;
		let moves = c.path.len();
		
		let mut path: Vec::<Move> = c.path.to_path().iter().map(|m| m.reverse()).clone().collect();
		path.reverse();
		
		if verbosity > 0 { println!("Selected level {}: depth {}, moves {}, path {}", idx, c.depth, moves, moves_to_string(&path)); }
		
		//TODO: move human to random (accessible) posn so first move is less obvious

		let mut level = Level::from_parts(base_level.get_title_str(), base_level.w, base_level.h, splevel.human_pos.clone(), splevel.get_data(&base_level));
		level.set_keyval("moves", &moves.to_string());
		level.set_keyval("depth", &c.depth.to_string());
		level.set_keyval("path", &moves_to_string(&path));
		level.place_human();
		levels.push(level);
	}
	
	levels
}


const DEF_MAX_MOVES: u16 = 200;
const DEF_MAX_DEPTH: u16 = 100;
const DEF_WIDTH: usize = 5;
const DEF_HEIGHT: usize = 5;
const DEF_BOX_DENSITY: u32 = 20;
const DEF_WALL_DENSITY: u32 = 20;
const DEF_VERBOSITY: u32 = 1;
const DEF_MAX_MAPS: usize = 1_200_000; // typically ~12gig of ram, for 16gig desktop


fn main() -> std::io::Result<()> {
	let args: Vec::<String> = std::env::args().collect();
	#[derive(PartialEq)]
	enum Mode { Help, Solve, Make };
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
				_ => {
					println!("First argument should be make or solve");
				}
			};
		} else if count >= 2 {
			let eq_idx = arg.find('=');
			if eq_idx.is_none() {
				println!("No equals symbol found in var {}", arg);
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

	if width > 15 || height > 15 || (width * height) > 120 {
		println!("ERROR: Maximum width is 15. Maximum height is 15. Maximum width*height is 120.");
		return Ok(());
	} 

	if width > 127 || height > 127 {
		println!("ERROR: Maximum width is 127. Maximum height is 127.");
		return Ok(());
	} 

	let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(0x0d47d47000000000_u64 + seed as u64);

	if mode == Mode::Help {
		println!("boxboppertool make [vars...]\nboxboppertool solve [vars...]\n");
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
			let solution = solve_level(&unsolved_levels[x], unsolved_levels[x].get_keyval("moves").parse::<u16>().expect("number->string->number failure!")+2, max_maps, &mut rng, verbosity); // probably don't need the +2
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
		output_str += &format!("time: {:.1}\n", solution.msecs/1000_f64);
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
	} else { // mode == solve
		// load level
		let level = if filename.len() > 0 {
			Level::from_file(&filename).expect("Unable to open specified file")
		} else {
			Level::from_builtin(builtin as usize).expect(&format!("Unable to open builtin level {}!", builtin))
		};
		
		println!("{}",level.to_string());

		let solution = solve_level(&level, max_moves, max_maps, &mut rng, verbosity);
		match solution {
			Some(_sol) => {
			},
			None => {
			}
		}
	}

	return Ok(());
}

