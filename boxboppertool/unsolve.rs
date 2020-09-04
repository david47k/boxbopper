// unsolve.rs: unsolve (create) a sokoban-style level

use boxbopperbase::{moves_to_string};
use boxbopperbase::level::{Level,CmpData};
use boxbopperbase::vector::{Move,ShrunkPath};

use rayon::prelude::*;
use std::rc::Rc;
use std::collections::BTreeSet;

use crate::pathnodemap::{PathNodeMap,PathMap,KeyMove,dedupe_equal_levels};

extern crate rand;
extern crate rand_chacha;

use rand::{Rng};


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


pub fn unsolve_level(base_level: &Level, max_depth: u16, max_maps: usize, rng: &mut rand_chacha::ChaCha8Rng, verbosity: u32) -> Vec::<Level> {
	let mut base_level = base_level.clone();
	base_level.clear_human();
	let base_level = &mut base_level;
	let base_map = PathMap::new_from_level(base_level);
	base_level.clear_boxxes();

	// A map is complete when the last box is pushed into place. So when unsolving, we need to start with the human
	// in the appropriate spot(s) they'd be after pushing the last box.
	// To do this, we unsolve once to find the appropriate spot(s), then re-solve to place the human and box in the final state.

	if verbosity > 1 { println!("finding final maps..."); }
	let mapsr: Vec<PathNodeMap> = vec![base_map].iter().map(|m| m.complete_map_unsolve(base_level) ).collect();
	let mapsr: Vec<PathMap> = mapsr.iter().flat_map(|map| map.apply_key_pulls() ).collect();
	let mapsr: Vec<PathNodeMap> = mapsr.iter().map(|m| m.complete_map_solve(base_level) ).collect();
	let mapsr: Vec<PathMap> = mapsr.iter().flat_map(|map| map.apply_key_pushes() ).collect();
	let mut mapsr: Vec<PathMap> = mapsr.iter().filter(|m| m.level.have_win_condition(base_level) ).cloned().collect();
	mapsr.iter_mut().for_each(|mut map| { 			// reset the move count
		map.path = ShrunkPath::new(); 
	});
	if verbosity > 1 { 
		println!("final maps found: {}", mapsr.len()); 
		for m in mapsr.iter() {
			println!("{}",m.level.to_level(base_level).to_string());
		}
	}
	let mut mapsr = Rc::new(mapsr);

	let mut non_contenders = BTreeSet::<CmpData>::new();
	let mut contenders = Vec::<PathMap>::new();	

	for count in 0..=(max_depth+1) {
		println!("--- Depth {:>2} ---", count);
		
		// complete the maps (finding keymoves as it goes)
		if verbosity > 1 { println!("completing  {:>7} maps", mapsr.len()); }
		let maps: Vec<PathNodeMap> = mapsr.par_iter().map(|m| m.complete_map_unsolve(base_level) ).collect();

		// apply key moves
		if verbosity > 1 { println!("collecting kms..."); }
		let todo_list: Vec<(&PathNodeMap,&KeyMove)> = maps.iter().flat_map(|m| m.key_moves.iter().map(|mv| (m,mv)).collect::<Vec::<(&PathNodeMap,&KeyMove)>>() ).collect();
		if verbosity > 1 { println!("applying kms..."); }
		let mut maps: Vec<PathMap> = todo_list.par_iter().map(|(m,mv)| PathMap::new_by_applying_key_pull(m,mv,count+1)).collect();
		
		// sort and deduplicate
		if verbosity > 1 { println!("deduping: before {:>7}", maps.len()); }
		dedupe_equal_levels(&mut maps);
		if verbosity > 1 { println!("deduping: after  {:>7}", maps.len()); }

		// mapsr --> contenders --> non-contenders	

		// keep top 20 mapsr, rest to non-contenders, but that means we gotta sort mapsr by depth/path length
		if verbosity > 1 { println!("sorting old maps"); }
		Rc::get_mut(&mut mapsr).unwrap().par_sort_unstable_by(|a,b| a.depth.partial_cmp(&b.depth).unwrap());
		let split_idx = if mapsr.len() > 20 { mapsr.len() - 20 } else { mapsr.len() };
		let mut top_20 = Rc::get_mut(&mut mapsr).unwrap().split_off(split_idx);
		// send rest of mapsr to non_contenders
		if non_contenders.len() < max_maps * 4 {
			mapsr.iter().for_each(|m| { non_contenders.insert(m.level.cmp_data); });
		} else {
			if verbosity > 0 { println!("--- Hit maximum old maps, not adding any more ---"); }
		}
		// don't need mapsr anymore
		std::mem::drop(mapsr);

		if verbosity > 1 { println!("keep top 20 contenders..."); }

		// save top20 to contenders
		contenders.append(&mut top_20);

		// keep top 20 contenders
		if contenders.len() > 20 {
			// save excess contenders to non-contenders
			let keep = contenders.split_off(contenders.len()-20);
			if non_contenders.len() < max_maps * 4 {
				contenders.iter().for_each(|m| { non_contenders.insert(m.level.cmp_data); });
			} else {
				if verbosity > 0 { println!("--- Hit maximum old maps, not adding any more ---"); }
			}
			contenders = keep;					// copy keep back
		}

		// remove from maps, anyhthing that is in non_contenders
		if verbosity > 1 { println!("deduping from n-c: before {:>7}", maps.len()); }
		maps.par_iter_mut().for_each(|m| if non_contenders.contains(&m.level.cmp_data) { m.flag = true; }); 
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
			while maps.len() > max_maps/8 {
				let mut i = 0;
				maps.retain(|_m| { i+=1; return i%2==1; } );
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

	// re-sort by depth -- maximise depth, otherwise equal
	contenders.par_sort_unstable_by(|a,b| a.depth.partial_cmp(&b.depth).unwrap() );

	let truncsize = 5;
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

		let mut level = splevel.to_level(base_level);
		level.set_keyval("moves", &moves.to_string());
		level.set_keyval("depth", &c.depth.to_string());
		level.set_keyval("path", &moves_to_string(&path));
		level.place_human();
		levels.push(level);
	}
	
	levels
}