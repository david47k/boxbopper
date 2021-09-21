// unsolve.rs: unsolve (create) a sokoban-style level

use boxbopperbase::{moves_to_string};
use boxbopperbase::level::{Level,CmpData};
use boxbopperbase::vector::{Move};

use rayon::prelude::*;
use std::rc::Rc;
use std::collections::BTreeMap;

use crate::solve::{task_splitter,task_splitter_mut,task_splitter_sort};

use crate::pathnodemap::{PathMap};

extern crate rand;
extern crate rand_chacha;

use rand::{Rng};

use bevy_tasks::{TaskPoolBuilder};


pub fn select_unique_n_from(count: usize, len: usize, rng: &mut rand_chacha::ChaCha8Rng) -> Vec::<usize> {
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


pub fn unsolve_level(base_level_in: &Level, max_depth: u16, max_maps: usize, rng: &mut rand_chacha::ChaCha8Rng, verbosity: u32, num_threads: usize) -> Vec::<Level> {
	let base_level1 = base_level_in.clear_human_cloned();
	let base_map = PathMap::new_from_level(&base_level1);
	let base_level = base_level1.clear_boxxes_cloned();

	let pool = TaskPoolBuilder::new()
	.thread_name("Box Bopper Tool Thread Pool".to_string())
	.num_threads(num_threads)
	.build();

	// A map is complete when the last box is pushed into place. So when unsolving, we need to start with the human
	// in the appropriate spot(s) they'd be after pushing the last box.
	// To do this, we unsolve once to find the appropriate spot(s), then re-solve to place the human and box in the final state.

	if verbosity > 1 { println!("finding final maps..."); }
	let vbm = vec![base_map];
	let mut maps1 = Vec::<PathMap>::new();
	vbm.iter().for_each(|m| m.complete_unsolve_2(&base_level, &mut maps1, 0));
	let mut maps2 = Vec::<PathMap>::new();
	maps1.iter().for_each(|m| m.complete_solve_2(&base_level, &mut maps2));
	let mut mapsr: Vec<PathMap> = maps2.iter().filter(|m| m.level.have_win_condition(&base_level) ).cloned().collect();
	mapsr.iter_mut().for_each(|map| { 			// reset the move count
		map.path.clear(); 
	});
	if verbosity > 1 { 
		println!("final maps found: {}", mapsr.len()); 
		for m in mapsr.iter() {
			println!("{}",m.level.to_level(&base_level).to_string());
		}
	}
	let mut mapsr = Rc::new(mapsr);

	let mut non_contenders = BTreeMap::<CmpData,u16>::new();
	let mut contenders = Vec::<PathMap>::new();	
	let mut contenders_2 = Vec::<PathMap>::new();
	let mut max_max_counter = 0;

	for count in 0..=(max_depth+1) {
		println!("--- Depth {:>2} ---", count);
		
		// Perform next key moves
		if verbosity > 1 { println!("performing next key moves..."); }
		let mut maps = task_splitter(&pool, num_threads, &mapsr, |maps_read: &[PathMap], mut maps_write: &mut Vec::<PathMap>| {
			maps_read.iter().for_each(|m| m.complete_unsolve_2(&base_level, &mut maps_write, count));		// perform next key moves
			//maps_write.retain(|m| m.path.len() < max_moves);										// filter out long moves
		});

		// Sort and deduplicate
		if verbosity > 1 { println!("deduping: before {:>7}", maps.len()); }
		maps = task_splitter_sort(&pool, num_threads, maps);
		if verbosity > 1 { println!("deduping: after  {:>7}", maps.len()); }

		// shuffle mapsr->contenders->contenders_2->non_contenders
		if verbosity > 1 { println!("keep top contenders..."); }
		if non_contenders.len() < max_maps * 4 {
			contenders_2.iter().for_each(|m| { 
				let pre = non_contenders.get(&m.level.cmp_data);
				if (pre.is_some() && *pre.unwrap() > m.path.len()) || pre.is_none() {
					non_contenders.insert(m.level.cmp_data, m.path.len()); 
				}
			});
		} else {
			if verbosity > 0 { println!("--- Hit maximum old maps, not adding any more ---"); }
		} 
		contenders_2 = contenders;
		contenders = mapsr.to_vec();
		
		// don't need mapsr anymore
		//std::mem::drop(mapsr);

		// Remove from maps anything that is in c2 AND we already found a shorter path
		if verbosity > 1 { println!("deduping using c2: before {:>7}", maps.len()); }
		/* task_splitter_mut(&pool, num_threads, maps, |ms| {
			for m in ms {
				let v = contenders_2.binary_search_by(|c2m| c2m.level.cmp_data.partial_cmp(&m.level.cmp_data).unwrap());
				if v.is_ok() {
					if contenders_2[v.unwrap()].path.len() <= m.path.len() {
						m.flag = true;
					}
				}
			}
		}); */

		maps.par_iter_mut().for_each(|m| {
			let v = contenders_2.binary_search_by(|c2m| c2m.level.cmp_data.partial_cmp(&m.level.cmp_data).unwrap());
			if v.is_ok() {
				if contenders_2[v.unwrap()].path.len() <= m.path.len() {
					m.flag = true;
				}
			}
		});
		maps.retain(|m| !m.flag);
		if verbosity > 1 { println!("deduping using c2: after  {:>7}", maps.len()); }

		// Remove from maps anything that is in c AND we already found a shorter path
		if verbosity > 1 { println!("deduping using c: before {:>7}", maps.len()); }
		maps.par_iter_mut().for_each(|m| {
			let v = contenders.binary_search_by(|cm| cm.level.cmp_data.partial_cmp(&m.level.cmp_data).unwrap());
			if v.is_ok() {
				if contenders[v.unwrap()].path.len() <= m.path.len() {
					m.flag = true;
				}
			}
		});
		maps.retain(|m| !m.flag);
		if verbosity > 1 { println!("deduping using c2: after  {:>7}", maps.len()); }		
		
		// Remove from maps anything that is in non_contenders AND we already found a shorter path
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

		// check if we've run out of options, if we have, then contenders is what we have
		if maps.len() == 0 {
			if verbosity > 0 { println!("-- No further moves possible --"); }
			if contenders.len() < 10 {
				contenders.append(&mut contenders_2);
			}
			break;
		}		

		// check if we've hit max depth, in which case we have maps / contenders
		if count > max_depth {
			if verbosity > 0 { println!("-- Hit depth limit --"); }
			contenders.append(&mut maps);
			if contenders.len() < 10 {
				contenders.append(&mut contenders_2);
			}
			break;
		}

		// check if we have waaaaaaaaaaay too many maps
		if maps.len() > max_maps/8 {
			if max_max_counter == 3 { // We've hit the limit too many times, it'll be a pain to solve
				println!("--- Hit maximum unsolve maps limit (4), finishing ---"); 
				
				contenders = maps;
				break;
			}
			println!("--- Hit maximum unsolve maps {}, reducing ---",max_maps/8); 
			while maps.len() > max_maps/8 {
				let mut i = 0;
				maps.retain(|_m| { i+=1; return i%2==1; } );	// These are all at same depth so we can just randomly reduce it
			}
			max_max_counter += 1;
		}

		mapsr = Rc::new(maps);
	}

	if verbosity > 1 { println!("Max depth was {}",max_depth); }
	if contenders.len() == 0 {
		println!("-- No maps to choose from! --");
		return Vec::<Level>::new();
	}

	if verbosity > 0 { print!("Contenders size {} -> ",contenders.len()); }

	// re-sort by depth -- maximise depth, maximise moves
	contenders.par_sort_unstable_by(|a,b| {
		let o = b.depth.partial_cmp(&a.depth).unwrap();
		if o == std::cmp::Ordering::Equal {
			return b.path.len().partial_cmp(&a.path.len()).unwrap();
		}
		o
	});

	let truncsize = 10;
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
		
		// TODO?: move human to random (accessible) posn so first move is less obvious
		// In practice, the human has usually pulled themselves in to a corner or something so we can't move anyway

		let mut level = splevel.to_level(&base_level);
		level.set_keyval("moves", &moves.to_string());
		level.set_keyval("depth", &c.depth.to_string());
		level.set_keyval("path", &moves_to_string(&path));
		levels.push(level);
	}
	
	levels
}