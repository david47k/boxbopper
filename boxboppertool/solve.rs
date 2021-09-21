// boxboppertool Copyright 2020-2021 David Atkinson
//
// solve.rs: solve a sokoban-style level

use boxbopperbase::level::{Level,CmpData};
use boxbopperbase::time::{get_time_ms};

use rayon::prelude::*;
use std::rc::Rc;
use std::collections::{BTreeMap};
use std::cmp::Ordering;
use itertools::Itertools;
use crate::pathnodemap::{PathMap};

extern crate rand;
extern crate rand_chacha;

use bevy_tasks::{TaskPool,TaskPoolBuilder};

pub fn task_splitter(pool: &TaskPool, spl_into: usize, from: &Vec::<PathMap>, func: impl Fn(&[PathMap], &mut Vec::<PathMap>) + Send + Copy + Sync) -> Vec::<PathMap> {
	// break up vecs
	let from_a = vec_slicer(from, spl_into);
	let mut to_a = vec_new_split_store(from.len() / spl_into + 1, spl_into);

	pool.scope(|s| {
		for i in 0..spl_into {
			unsafe { // actually safe, as we don't use overlapping indices
				let from_sm: &[PathMap] = *(from_a.get_unchecked(i) as *const _);
				let to_sm = &mut *(to_a.get_unchecked_mut(i) as *mut _);
				s.spawn( async move {
					func(&from_sm, to_sm);
				})
			}
		}
	});

	let maps = vec_unslice(to_a);
	maps
}

pub fn task_splitter_mut(pool: &TaskPool, spl_into: usize, mut maps: Vec::<PathMap>, func: impl Fn(&mut [PathMap]) + Send + Copy + Sync) -> Vec::<PathMap> {
	// break up vecs
	let mut maps_a = vec_slicer_mut(&mut maps, spl_into);

	pool.scope(|s| {
		for i in 0..spl_into {
			unsafe { // actually safe, as we don't use overlapping indices
				let maps_sm: &mut &mut [PathMap] = &mut *(maps_a.get_unchecked_mut(i) as *mut _);
				s.spawn( async move {
					func(maps_sm);
				})
			}
		}
	});

	// no unslice required, as we used mutable references :)
	maps
}

// Faster than rayon::par_sort_unstable
pub fn task_splitter_sort(pool: &TaskPool, spl_into: usize, mut maps: Vec::<PathMap>) -> Vec::<PathMap> {
	// break up vecs
	let mut maps_a = vec_slicer_mut(&mut maps, spl_into);

	pool.scope(|s| {
		for i in 0..spl_into {
			unsafe { // actually safe, as we don't use overlapping indices
				let maps_sm: &mut &mut [PathMap] = &mut *(maps_a.get_unchecked_mut(i) as *mut _);
				s.spawn( async move {
					maps_sm.sort_unstable_by(|a: &PathMap, b: &PathMap| {
						let ord = a.level.cmp_data.partial_cmp(&b.level.cmp_data).unwrap();
						if ord == Ordering::Equal {
							if a.path.len() < b.path.len() {
								return Ordering::Less;
							}
							if a.path.len() > b.path.len() {
								return Ordering::Greater;
							}
						}
						ord
					})
				})
			}
		}
	});

	fn pm_cmp_lt (a: & &mut PathMap, b: & &mut PathMap) -> bool {
		let ord = a.level.cmp_data.partial_cmp(&b.level.cmp_data).unwrap();
		if ord == Ordering::Equal {
			if a.path.len() < b.path.len() {
				return true;
			}
			return false;
		}
		return ord == Ordering::Less;
	}

	let mut maps: Vec::<PathMap> = maps_a.into_iter().map(|x| x).kmerge_by(pm_cmp_lt).map(|x| x.to_owned()).collect();
	maps.dedup_by(|a,b| a.level.cmp_data == b.level.cmp_data); // it keeps the first match for each level (sorted to be smallest moves)
	maps
}

// borrows the provided vec, and provides a vec of slices as output
pub fn vec_slicer(from: &Vec::<PathMap>, spl_into: usize) -> Vec::<&[PathMap]> {
	let size = from.len() / spl_into;
	let mut out: Vec::<&[PathMap]>;
	out = Vec::<&[PathMap]>::with_capacity(spl_into);
	if from.len() < spl_into { 
		out.push( &from[..] );
		for _i in 1..spl_into  {
			out.push( &from[0..0] );
		}
	} else {
		let mut count = 0;
		for _i in 1..spl_into  {
			out.push( &from[count..(count+size)] );
			count += size;
		}
		out.push( &from[count..] );
	}
	out
}

// borrows the provided vec, and provides a vec of mutable slices as output
pub fn vec_slicer_mut(from: &mut Vec::<PathMap>, spl_into: usize) -> Vec::<&mut [PathMap]> {
	// assert(spl_into > 0);
	let size = from.len() / spl_into;
	let mut out: Vec::<&mut [PathMap]> = Vec::<&mut [PathMap]>::with_capacity(spl_into);
	unsafe { // actually safe, as we don't use overlapping indices
		if from.len() < spl_into { 
			let from_sm: &mut [PathMap] = &mut *(from.get_unchecked_mut(..) as *mut _);
			out.push( from_sm );
			for _i in 1..spl_into  {
				let from_sm: &mut [PathMap] = &mut *(from.get_unchecked_mut(0..0) as *mut _);
				out.push( from_sm );
			}
		} else {
			let mut count = 0;
			for _i in 1..spl_into  {
				let from_sm: &mut [PathMap] = &mut *(from.get_unchecked_mut(count..(count+size)) as *mut _);
				out.push( from_sm );
				count += size;
			}
			out.push( &mut from[count..] );
		}
	}
	out
}

pub fn vec_new_split_store(size: usize, spl_into: usize) -> Vec::<Vec::<PathMap>> {
	let mut out = Vec::<Vec::<PathMap>>::with_capacity(spl_into);
	for _i in 0..spl_into  {
		out.push( Vec::<PathMap>::with_capacity(size) );
	}
	out
}

pub fn vec_unslice(mut from: Vec::<Vec::<PathMap>>) -> Vec::<PathMap> {
	for i in 1..from.len() {
		unsafe {
			// actually safe because we aren't accessing two identical indicies (it'll always be 0 and 1+)
			let src = &mut *(from.get_unchecked_mut(i) as *mut _);
			from[0].append(src);
		}
	}
	return from.swap_remove(0);
}


#[derive(Clone)]
pub struct Solution {
	pub moves: u16,
	pub depth: u16,
	pub secs: f64,
	pub path: String
}


pub fn solve_level(base_level_in: &Level, max_moves_requested: u16, max_maps: usize, verbosity: u32, num_threads: usize) -> Option<Solution> {
	let mut max_moves = max_moves_requested+1;
	let base_level1 = base_level_in.clear_human_cloned();
	let base_map = PathMap::new_from_level(&base_level1);
	let base_level = base_level1.clear_boxxes_cloned();
	let mut non_contenders = BTreeMap::<CmpData,u16>::new();

	let mut bvec = Vec::new();
	bvec.push(base_map);
	let mut mapsr = Rc::new(bvec);
	
	let pool = TaskPoolBuilder::new()
	.thread_name("Box Bopper Tool Thread Pool".to_string())
	.num_threads(num_threads)
	.build();

	let mut have_solution = false;
	struct BestSolution {
		s: String,
		depth: u16,
	}
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
			//mapsr.par_iter().for_each(|m| { non_contenders.insert(m.level.cmp_data, m.path.len()); });
			non_contenders.par_extend(mapsr.par_iter().map(|m| (m.level.cmp_data, m.path.len()) ));
		} else {
			if verbosity > 0 { println!("--- Old maps hit max_maps limit, not adding more ---"); }
		}

		// Perform next key moves
		if verbosity > 1 { println!("performing next key moves..."); }
		let mut maps = task_splitter(&pool, num_threads, &mapsr, |maps_read: &[PathMap], mut maps_write: &mut Vec::<PathMap>| {
			maps_read.iter().for_each(|m| m.complete_solve_2(&base_level, &mut maps_write));		// perform next key moves
			maps_write.retain(|m| m.path.len() < max_moves);										// filter out long moves
		});

		// Sort and deduplicate
		if depth >= 2 { 
			if verbosity > 1 { println!("deduping: before {:>7}", maps.len()); }
			maps = task_splitter_sort(&pool, num_threads, maps);
			if verbosity > 1 { println!("deduping: after  {:>7}", maps.len()); }
		} 

		// Remove from maps anything that is in non_contenders AND our path is equal/longer
		if verbosity > 1 { println!("deduping using n-c: before {:>7}", maps.len()); }		
		maps = task_splitter_mut(&pool, num_threads, maps, |maps: &mut [PathMap]| {
			for m in maps {
				let v = non_contenders.get(&m.level.cmp_data);
				if v.is_some() {
					if *v.unwrap() <= m.path.len() {
						m.flag = true;
					}
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
