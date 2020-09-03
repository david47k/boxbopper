// PathNode, PathMap, PathNodeMap and family
// Used for creating and solving levels

use std::cmp::Ordering;
use rayon::prelude::*;

use boxbopperbase::{Obj};
use boxbopperbase::level::{Level,SpLevel};
use boxbopperbase::vector::{Vector,Move,ALLMOVES,ShrunkPath};

#[derive(Clone,Copy)]
pub struct PathNode {
	pt: Vector,
	prev_node_idx: u16,
	move_taken: Option<Move>, // what move we took to get here, used to determine movelist when solution found
}

#[derive(Clone,Copy)]
pub struct KeyMove {
	pn: PathNode,		// where human is just before pushing boxx
	move_dir: Move,		// direction to move to push boxx (or direction we are pulling box in)
}

#[derive(Clone)]
pub struct PathNodeMap {
	pub nodes: Vec::<PathNode>,
	pub key_moves: Vec::<KeyMove>,	
	pub map: PathMap,
}

#[derive(Clone)]
pub struct PathMap {
	pub level: SpLevel,
	pub path: ShrunkPath,
	pub depth: u16,
	pub flag: bool,
}


impl PathMap {
	pub fn new_from_level(level: &Level) -> PathMap {
		PathMap {
			level: SpLevel::from_level(level),
			path: ShrunkPath::new(),
			depth: 0,
			flag: false,
		}
	}
	pub fn new_by_applying_key_push(pnm: &PathNodeMap, km: &KeyMove) -> PathMap { 	// after we complete a map, we need to take a key move and start again
		// do we read from pnm.map, or from map_b??? which is faster?
		
		let mut map_b = pnm.map.clone();
				
		// new human point
		let np = km.pn.pt.add_dir(&km.move_dir);
		
		// check destination point
		if pnm.map.level.is_boxx_at_pt(&np) {
			let boxx_pt = &np.add_dir(&km.move_dir);
			let is_clear = !pnm.map.level.is_boxx_at_pt(&boxx_pt);
			if is_clear {
				map_b.level.set_boxx_at_pt(&boxx_pt);
			} else {
				panic!("trying to push boxx into unexpected obj");
			}
		
			// We pushed the boxx
			map_b.level.clear_boxx_at_pt(&np);
		} else {
			panic!("Human not allowed there! (No boxx to push!)");
		}

		map_b.level.set_human_pos(&np);				// move human
			
		let bm = pnm.backtrace_moves(&km.pn);
		for m in bm {
			map_b.path.push(&m);
		}
		map_b.path.push(&km.move_dir);
		
		map_b
	}
	pub fn to_pnm(&self) -> PathNodeMap {		// this one clones across our data
		let initial_pn = PathNode {
			pt: Vector(self.level.cmp_data.human_x.into(), self.level.cmp_data.human_y.into()),
			move_taken: None,
			prev_node_idx: 0,
		};
		let mut nodes = Vec::<PathNode>::with_capacity(32);
		nodes.push(initial_pn);
		PathNodeMap {
			map: PathMap {
				level: self.level.clone(),
				path: self.path.clone(),
				depth: 0,
				flag: false,
			},
			nodes: nodes,
			key_moves: Vec::<KeyMove>::with_capacity(16),
		}
	}
	pub fn new_by_applying_key_pull(pnm: &PathNodeMap, km: &KeyMove, depth: u16) -> PathMap { 	// after we complete a map, we need to take a key move and start again
		// do we read from pnm.map, or from map_b??? which is faster?

		let mut map_b = pnm.map.clone();
		map_b.depth = depth;
				
		// remove old boxx
		let pull_from_pt = km.pn.pt.add_dir(&km.move_dir.reverse());
		let is_boxx = pnm.map.level.is_boxx_at_pt(&pull_from_pt);
		if is_boxx {
			map_b.level.clear_boxx_at_pt(&pull_from_pt);
		} else {
			panic!("Key pull doesn't seem to be moving a boxx!");
		}

		// place new boxx
		let pull_to_pt = &km.pn.pt;
		let is_clear = !pnm.map.level.is_boxx_at_pt(&pull_from_pt);
		if is_clear {
			map_b.level.set_boxx_at_pt(pull_to_pt);
		} else {
			panic!("Key pull seems to be moving boxx into something weird!");
		}
		
		// new human point
		let np = km.pn.pt.add_dir(&km.move_dir);
		map_b.level.set_human_pos(&np);

		let bm = pnm.backtrace_moves(&km.pn);
		for m in bm {
			map_b.path.push(&m);
		}
		map_b.path.push(&km.move_dir);

		map_b
	}
	pub fn complete_map_solve(&self, base_level: &Level) -> PathNodeMap {
		let mut pnm = self.to_pnm();					// we want complete_map to clone from self
		let mut tail_nodes = Vec::<u16>::with_capacity(64);
		tail_nodes.push(0);
		let mut new_tail_nodes = Vec::<PathNode>::with_capacity(64);	// somewhere to store new tail nodes
		while tail_nodes.len() != 0 {			// check if map is complete
			new_tail_nodes.clear();
			for tnidx in tail_nodes.iter() {							// for each tail node
				let tnode = &pnm.nodes[(*tnidx) as usize];
				let pt = tnode.pt;									
				for movedir in ALLMOVES.iter() {							// for each possible move
					let npt = pt.add_dir(&movedir);							// what is in this direction? let's find out
					if !base_level.vector_in_bounds(&npt) { continue; }
					if pnm.map.level.is_boxx_at_pt(&npt) {
						// What's past the boxx? We can push into Space and Hole.
						let bnpt = &pt.add_dir2(&movedir);
						//if !base_level.vector_in_bounds(bnpt) { continue; }
						let nobj = pnm.map.level.get_obj_at_pt_checked(bnpt, base_level);
						if nobj == Obj::Space || nobj == Obj::Hole {
							// Obj::Space | Obj::Hole => { 
							// yep, its a keymove, save key move.. but before we do, make sure it isn't a double boxx situation or in our noboxx list
							// TODO: see if double_Boxx_situation is too slow (after optimising it)
							if !base_level.in_noboxx_pts(bnpt) && !self.double_boxx_situation(pt,*movedir,base_level) {
								let km = KeyMove {
									pn: tnode.clone(),
									move_dir: *movedir,
								};
								pnm.key_moves.push(km);
							}
						}
					} else if base_level.get_obj_at_pt(&npt) != Obj::Wall {											
						// first check this point isn't already in our list!!!						
						let mut ok = true;
						for n in pnm.nodes.iter() {
							if n.pt == npt { ok = false; break; }
						}
						for n in new_tail_nodes.iter() {
							if n.pt == npt { ok = false; break; }
						}
						if !ok { continue; }

						// yep, we can move here, make a new tail node
						let pn = PathNode {
							pt: npt.clone(),
							move_taken: Some(*movedir),
							prev_node_idx: *tnidx,
						};
						new_tail_nodes.push(pn);
					}
				}	
			}
	
			// append new tail nodes to nodes and tail nodes
			tail_nodes.clear();
			for n in new_tail_nodes.iter() {
				pnm.nodes.push(*n);
				tail_nodes.push((pnm.nodes.len()-1) as u16);
			}
		}
		pnm
	}
	pub fn complete_map_unsolve(&self, base_level: &Level) -> PathNodeMap {
		let mut pnm = self.to_pnm();					// we want complete_map to clone from self
		let mut tail_nodes = Vec::<u16>::with_capacity(64);
		tail_nodes.push(0);
		let mut new_tail_nodes = Vec::<PathNode>::with_capacity(64);	// somewhere to store new tail nodes
		while tail_nodes.len() != 0 {			// check if map is complete
			new_tail_nodes.clear();
			for tnidx in tail_nodes.iter() {							// for each tail node
				let tnode = &pnm.nodes[*tnidx as usize];
				let pt = tnode.pt;									
				for movedir in ALLMOVES.iter() {							// for each possible move
					let npt = pt.add_dir(&movedir);							// what is in this direction? let's find out	
					if !base_level.vector_in_bounds(&npt) { continue; }
					if pnm.map.level.is_boxx_at_pt(&npt) {
						// What's in our reverse direction? We can pull into Space and Hole.
						let bnpt = &pt.add_dir(&movedir.reverse());
						//if !base_level.vector_in_bounds(bnpt) { continue; }
						let nobj = pnm.map.level.get_obj_at_pt_checked(bnpt, base_level);
						if nobj == Obj::Space || nobj == Obj::Hole {
							// Obj::Space | Obj::Hole => { 
							// yep, its a keypull, save key move.. 
							let km = KeyMove {
								pn: tnode.clone(),
								move_dir: movedir.clone().reverse(),
							};
							pnm.key_moves.push(km);
						}
					} else if base_level.get_obj_at_pt(&npt) != Obj::Wall {
						// first check this point isn't already in our list!!!						
						let mut ok = true;
						for n in pnm.nodes.iter() {
							if n.pt == npt { ok = false; break; }
						}
						for n in new_tail_nodes.iter() {
							if n.pt == npt { ok = false; break; }
						}
						if !ok { continue; }

						// yep, we can move here, make a new tail node
						let pn = PathNode {
							pt: npt.clone(),
							move_taken: Some(*movedir),
							prev_node_idx: *tnidx,
						};
						new_tail_nodes.push(pn);
					}
				}	
			}

			// append new tail nodes to nodes and tail nodes
			tail_nodes.clear();
			for n in new_tail_nodes.iter() {
				pnm.nodes.push(*n);
				tail_nodes.push((pnm.nodes.len()-1) as u16);
			}
		}		
		pnm
	}
	pub fn double_boxx_situation(&self, human_pos: Vector, pushdir: Move, base_level: &Level) -> bool {
		// checks for a situation where we would be pushing the boxx next to another boxx against a wall and getting ourselves stuck
		//         a = anything, h = human, pushdir = right, * = boxx, # = wall, ' ' = space, only need row 1 or 3 not both
		//  aa*#	matchB
		//  h* #	matchA
		//  aa*#	matchB
		// test 1 (horizontal in pushdir direction): [ Obj::Boxx, Obj::Space, Obj::Wall ]
		// test 2: above OR below (either or both, +1 in pushdir direction than above): [ Obj::Boxx, Obj::Wall ]
		// 
		// this method improves solution time by about 4-5%

		const MATCH_A: [Obj; 3] = [Obj::Boxx, Obj::Space, Obj::Wall];			
		// let _match0b = [Obj::BoxxInHole, Obj::Space, Obj::Wall];		// too slow
		
		const MATCH_B: [Obj; 2] = [Obj::Boxx, Obj::Wall];				
		// let _match1b = [Obj::BoxxInHole, Obj::Wall];					// too slow

		let pushv = pushdir.to_vector();

		let line0 = [ self.level.get_obj_at_pt_checked(&human_pos.add(&pushv), base_level),
					  self.level.get_obj_at_pt_checked(&human_pos.add(&pushv.mul(2)), base_level),
					  self.level.get_obj_at_pt_checked(&human_pos.add(&pushv.mul(3)), base_level) ];

		let hpadd2 = human_pos.add(&pushv.mul(2));
		let hpadd3 = human_pos.add(&pushv.mul(3));

		let line1 = [ self.level.get_obj_at_pt_checked(&hpadd2.add(&pushv.rotl()), base_level),
					  self.level.get_obj_at_pt_checked(&hpadd3.add(&pushv.rotl()), base_level) ];
		let line2 = [ self.level.get_obj_at_pt_checked(&hpadd2.add(&pushv.rotr()), base_level),
					  self.level.get_obj_at_pt_checked(&hpadd3.add(&pushv.rotr()), base_level) ];

		line0 == MATCH_A && ( line1 == MATCH_B || line2 == MATCH_B )
		
		// line0 == match0 && ( line1 == match1[0] || line2 == match1[0] || line1 == match1[1] || line2 == match1[1] ) // slows us down by 2.5%
		// line0 == match0 && ( contains_only(&match1, &line1) || contains_only(&match1, &line2) ) // too slow
	}	
}

impl PathNodeMap {
	pub fn apply_key_pushes(&self) -> Vec<PathMap> {			// avoid using these as they are slow
		let mut nmaps = Vec::<PathMap>::with_capacity(self.key_moves.len());
		for km in &self.key_moves {	
			nmaps.push(PathMap::new_by_applying_key_push(self, &km));
		}
		nmaps
	}
	pub fn apply_key_pulls(&self) -> Vec<PathMap> {				// avoid using these as they are slow
		let mut nmaps = Vec::<PathMap>::with_capacity(self.key_moves.len());
		for km in &self.key_moves {	
			nmaps.push(PathMap::new_by_applying_key_pull(self, &km, 0));
		}
		nmaps
	}
	pub fn backtrace_moves(&self, pn: &PathNode) -> Vec::<Move> {
		let mut path = Vec::<Move>::with_capacity(32);
		// start at pn and work backwards
		let mut pnr = pn;
		loop {
			if pnr.move_taken.is_some() {
				path.push(pnr.move_taken.unwrap());
				if pnr.prev_node_idx == 0 {
					let m = &self.nodes[0].move_taken;
					if m.is_some() { path.push(m.unwrap()); }
					break;
				}
				pnr = &self.nodes[pnr.prev_node_idx as usize];
			} else {
				break;
			}
		}
		path.reverse();
		path
	}
}


pub fn dedupe_equal_levels(maps: &mut Vec::<PathMap>) {
	maps.par_sort_unstable_by(|a,b| {
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
	});
	maps.dedup_by(|a,b| a.level.cmp_data == b.level.cmp_data); // it keeps the first match for each level (sorted to be smallest moves)
}