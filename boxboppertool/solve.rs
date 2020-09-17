// solve.rs: solve a sokoban-style level

use boxbopperbase::level::{Level,CmpData};
use boxbopperbase::time::{get_time_ms};

use rayon::prelude::*;
use std::rc::Rc;
use std::collections::{BTreeMap};

use crate::pathnodemap::{PathNodeMap,PathMap,KeyMove,dedupe_equal_levels};

extern crate rand;
extern crate rand_chacha;


#[derive(Clone)]
pub struct Solution {
	pub moves: u16,
	pub depth: u16,
	pub secs: f64,
	pub path: String
}


pub fn solve_level(base_level_in: &Level, max_moves_requested: u16, max_maps: usize, verbosity: u32) -> Option<Solution> {
	let mut max_moves = max_moves_requested+1;
	let base_level1 = base_level_in.clear_human_cloned();
	let base_map = PathMap::new_from_level(&base_level1);
	let base_level = base_level1.clear_boxxes_cloned();
	let mut non_contenders = BTreeMap::<CmpData,u16>::new();

	let mut bvec = Vec::new();
	bvec.push(base_map);
	let mut mapsr = Rc::new(bvec);
	
	let mut have_solution = false;
	struct BestSolution {
		s: String,
		depth: u16,
	};
	let mut best_solution = BestSolution { s: String::new(), depth: 0 };
	let mut depth: u16 = 0;
	
	let msecs0 = get_time_ms();

	while depth < max_moves {
		if verbosity > 0 { println!("-- Depth {:>2} --", depth); }

		// Check for level complete / having solution
		if verbosity > 1 { println!("solution check..."); }
		mapsr.iter().filter(|m| m.level.have_win_condition(&base_level)).for_each(|m| {
			if m.path.len() < max_moves {
				have_solution = true;
				max_moves = m.path.len();
				best_solution.depth = depth;
				best_solution.s = format!("{}", &m.path.to_string());
				if verbosity > 0 { 
					println!("-- Solution found in {} moves --", m.path.len());
				}
			}
		});

		// We have to store number of moves, because higher depth can have less moves
		if verbosity > 1 { println!("adding {} old maps to non-contenders...", mapsr.len()); }
		if non_contenders.len() < max_maps * 4 {
			//mapsr.iter().for_each(|m| { non_contenders.insert(m.level.cmp_data, m.path.len()); });
			non_contenders.par_extend(mapsr.par_iter().map(|m| (m.level.cmp_data, m.path.len()) ));
		} else {
			if verbosity > 0 { println!("--- Old maps hit max_maps limit, not adding more ---"); }
		}
		
		// Complete the maps, converting from PathMap into PathNodeMap
		if verbosity > 1 { println!("completing  {:>7} maps", mapsr.len()); }
		let maps: Vec<(PathNodeMap,&PathMap)> = mapsr.par_iter().map(|m| (m.complete_map_solve(&base_level),m) ).collect(); // collect_into_vec doesn't seem to be any faster

		// Free up memory used by the vec in mapsr
		// std::mem::drop(mapsr); // 2.76, eof 4.79 -> 0, 5.7: commenting out the drop improves speed 1.5% according to VS, but is less memory-friendly

		// Apply key moves
		if verbosity > 1 { println!("collecting kms..."); }
		let todo_list: Vec<(&PathNodeMap,&PathMap,&KeyMove)> = maps.iter().flat_map(|(pnm,pm)| pnm.key_moves.iter().map(|mv| (pnm,*pm,mv)).collect::<Vec::<(&PathNodeMap,&PathMap,&KeyMove)>>() ).collect();
		if verbosity > 1 { println!("applying kms..."); }
		let mut maps: Vec<PathMap> = todo_list.par_iter().map(|(pnm,pm,km)| PathMap::new_by_applying_key_push(pnm,pm,km)).collect();

		// Filter out the long paths
		if verbosity > 1 { println!("pruning long paths..."); }
		maps.retain(|m| m.path.len() < max_moves);

		// Sort and deduplicate
		if depth >= 2 { 
			if verbosity > 1 { println!("deduping: before {:>7}", maps.len()); }
			dedupe_equal_levels(&mut maps);
			if verbosity > 1 { println!("deduping: after  {:>7}", maps.len()); }
		} 

		// Remove from maps anything that is in non_contenders AND our path is equal/longer
		if verbosity > 1 { println!("deduping using n-c: before {:>7}", maps.len()); }
		maps.par_iter_mut().for_each(|m| {
			let v = non_contenders.get(&m.level.cmp_data);
			if v.is_some() {
				if *v.unwrap() <= m.path.len() {
					m.flag = true;
				}
			}
		});
		maps.retain(|m| !m.flag);
		if verbosity > 1 { println!("deduping using n-c: after  {:>7}", maps.len()); }

		// Check if we've exhausted the search space
		if maps.len() == 0 {
			if verbosity > 0 { println!("-- No more maps to check --"); }
			break;
		}

		// Check if we've hit max_maps (our memory/resource limit)
		if maps.len() > max_maps {
			println!("--- Hit maximum maps ({}) ---",max_maps);
			break;
		}
		
		// Loop and check the next depth
		mapsr = Rc::new(maps);
		depth += 1;
	}

	if have_solution {
		let sol = best_solution;		
		if verbosity > 0 { 
			println!("-- Best solution --");
			println!("Solution in {} moves: {}",max_moves, sol.s);
		}
		return Some(Solution {
			secs: (get_time_ms() - msecs0) / 1000_f64,
			moves: max_moves,
			depth: sol.depth,
			path: sol.s.to_string(),
		});
	} else {
		let ms = max_moves;
		if verbosity > 0 {
			println!("-- No solution found --");
			if ms > 1 { println!("Max moves was {}",ms-1); }
		}
		return None;
	}

}
