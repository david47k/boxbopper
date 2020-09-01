// solve.rs: solve a sokoban-style level

use boxbopperbase::level::{Level};
use boxbopperbase::time::{get_time_ms};

use rayon::prelude::*;
use std::sync::{Arc,Mutex};
use std::sync::atomic::Ordering as AtomicOrdering;
use std::sync::atomic::*;
use std::rc::Rc;

use crate::pathnodemap::{PathNodeMap,PathMap,KeyMove,dedupe_equal_levels};

extern crate rand;
extern crate rand_chacha;

use rand::{Rng};


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
	let base_map = PathMap::new_from_level(base_level);

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
		mapsr.par_iter().filter(|m| m.level.have_win_condition(base_level)).for_each(|m| {
			if m.path.len() < max_moves.load(AtomicOrdering::SeqCst) {
				have_solution.store(true, AtomicOrdering::SeqCst);
				max_moves.store(m.path.len(), AtomicOrdering::SeqCst);
				best_sol_depth.store(depth, AtomicOrdering::SeqCst);
				let mut solstr = best_solution_str.lock().unwrap();
				*solstr = format!("{}", &m.path.to_string());
				if verbosity > 0 { 
					println!("-- Solution found in {} moves --", m.path.len());
				}
			}
		});

		// complete the maps, converting from type B into type A...
		if verbosity > 1 { println!("completing  {:>7} maps", mapsr.len()); }
		let maps: Vec<PathNodeMap> = mapsr.par_iter().map(|m| m.complete_map_solve(base_level) ).collect(); // collect_into_vec doesn't seem to be any faster

		// apply key moves
		if verbosity > 1 { println!("collecting kms..."); }
		let todo_list: Vec<(&PathNodeMap,&KeyMove)> = maps.iter().flat_map(|m| m.key_moves.iter().map(|mv| (m,mv)).collect::<Vec::<(&PathNodeMap,&KeyMove)>>() ).collect();
		if verbosity > 1 { println!("applying kms..."); }
		let mut maps: Vec<PathMap> = todo_list.par_iter().map(|(m,mv)| PathMap::new_by_applying_key_push(m,mv)).collect();

		// filter out the long paths
		if verbosity > 1 { println!("pruning long paths..."); }
		let ms = max_moves.load(AtomicOrdering::SeqCst);
		maps.retain(|m| m.path.len() < ms);

		// sort and deduplicate
		if depth >= 2 { 
			if verbosity > 1 { println!("deduping: before {:>7}", maps.len()); }
			dedupe_equal_levels(&mut maps);
			if verbosity > 1 { println!("deduping: after  {:>7}", maps.len()); }
		} 

		// Shrink mapsr, add it to non-contenders, sort, dedupe current maps against non-contenders
		if verbosity > 1 { println!("Adding {} old maps to non-contenders...", mapsr.len()); }
		let mut data: Vec::<u128> = mapsr.iter().map(|m| m.level.cmp_data).collect(); 			// par_iter doesn't seem to make a difference
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
			if ms > 1 { println!("Max moves was {}",ms-1); }
		}
		return None;
	}

}
