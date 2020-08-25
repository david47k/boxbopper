// PathNodeMap and family
// Used for creating and solving levels

use std::sync::{Arc};

use boxbopperbase::{Obj};
use boxbopperbase::level::{Level,SpLevel};
use boxbopperbase::vector::{Vector,Move,ALLMOVES};

#[derive(Clone,Copy)]
pub struct PathNode {
	pt: Vector,
	pub steps: u32,
	move_taken: Option<Move>, // what move we took to get here, used to determine movelist when solution found
	prev_node_idx: usize,
}

#[derive(Clone,Copy)]
pub struct KeyMove {
	pn: PathNode,		// where human is just before pushing boxx
	move_dir: Move,		// direction to move to push boxx (or direction we are pulling box in)
}

#[derive(Clone)]
pub struct PathNodeMap {
	base_level: Arc::<Level>,
	pub level: SpLevel,
	pub nodes: Vec::<PathNode>,
	tail_nodes: Vec::<usize>,
	key_moves: Vec::<KeyMove>,
	pub path: Vec::<Move>,
	pub depth: u32,
	pub contender_flag: bool,
}

impl PathNodeMap {
	pub fn new_from_level(level: &Arc<Level>) -> PathNodeMap {				// start the game this way
		let mut map = PathNodeMap {
			base_level: level.clone(),
			level: SpLevel::new_from_level(&(level.clone())),
			nodes: Vec::<PathNode>::with_capacity(64),
			tail_nodes: Vec::<usize>::with_capacity(32),
			key_moves: Vec::<KeyMove>::with_capacity(16),
			path: Vec::<Move>::with_capacity(64),
			contender_flag: false,
			depth: 0,
		};
		map.nodes.push(PathNode {
			pt: level.human_pos.clone(),
			steps: 0,
			move_taken: None,
			prev_node_idx: 0,
		});
		map.tail_nodes.push(0);
		map
	}
	pub fn complete_map_solve(&self) -> PathNodeMap {
		let mut map = self.clone();
		while !map.is_map_complete() {
			map.step_solve();
		}
		map
	}
	pub fn complete_map_unsolve(&self) -> PathNodeMap {
		let mut map = self.clone();
		while !map.is_map_complete() {
			map.step_unsolve();
		}		
		map
	}
	pub fn step_solve(&mut self) { 										// steps tail nodes forwards one		
		let mut new_tail_nodes = Vec::<PathNode>::with_capacity(32);	// somewhere to store new tail nodes
		
		for tnidx in self.tail_nodes.iter() {							// for each tail node
			let tnode = &self.nodes[*tnidx];
			for movedir in ALLMOVES.iter() {							// for each possible move
				let pt = tnode.pt;									
				let npt = pt.add(&movedir.to_vector());					// what is in this direction? let's find out	
				match self.level.get_obj_at_pt(&npt) {
					Obj::Space | Obj::Hole => {
						// first check this point isn't already in our list!!!						
						let mut ok = true;
						for n in self.nodes.iter() {
							if n.pt.eq(&npt) { ok = false; break; }
						}
						for n in new_tail_nodes.iter() {
							if n.pt.eq(&npt) { ok = false; break; }
						}
						if !ok { continue; }

						// yep, we can move here, make a new tail node
						let pn = PathNode {
							pt: npt.clone(),
							steps: tnode.steps + 1,
							move_taken: Some(*movedir),
							prev_node_idx: *tnidx,
						};
						new_tail_nodes.push(pn);
					}
					Obj::Boxx | Obj::BoxxInHole => { 
						// What's past the boxx? We can push into Space and Hole (and HumanInHole cos we'll be out of the way), nothing else.
						let bnpt = &pt.add(&movedir.to_vector().double());
						match self.level.get_obj_at_pt(bnpt) {
							Obj::Space | Obj::Hole | Obj::Human | Obj::HumanInHole => { 
								// yep, its a keymove, save key move.. but before we do, make sure it isn't a double boxx situation or in our noboxx list
								if !self.base_level.in_noboxx_pts(*bnpt) && !self.double_boxx_situation(pt,*movedir) {
									let km = KeyMove {
										pn: tnode.clone(),
										move_dir: *movedir,
									};
									self.key_moves.push(km);
								}
							},
							_ => {} // can't push the boxx				
						}
					}
					_ => {} // not a move we can take
				};
			}	
		}

		// append new tail nodes to nodes and tail nodes
		self.tail_nodes.clear();
		for n in new_tail_nodes {
			self.nodes.push(n);
			self.tail_nodes.push(self.nodes.len()-1);
		}
	}
	pub fn step_unsolve(&mut self) { 									// steps tail nodes forwards one		
		let mut new_tail_nodes = Vec::<PathNode>::with_capacity(32);	// somewhere to store new tail nodes
		
		for tnidx in self.tail_nodes.iter() {							// for each tail node
			let tnode = &self.nodes[*tnidx];
			for movedir in ALLMOVES.iter() {							// for each possible move
				let pt = tnode.pt;									
				let npt = pt.add(&movedir.to_vector());					// what is in this direction? let's find out	
				match self.level.get_obj_at_pt(&npt) {
					Obj::Space | Obj::Hole => {
						// first check this point isn't already in our list!!!						
						let mut ok = true;
						for n in self.nodes.iter() {
							if n.pt.eq(&npt) { ok = false; break; }
						}
						for n in new_tail_nodes.iter() {
							if n.pt.eq(&npt) { ok = false; break; }
						}
						if !ok { continue; }

						// yep, we can move here, make a new tail node
						let pn = PathNode {
							pt: npt.clone(),
							steps: tnode.steps + 1,
							move_taken: Some(*movedir),
							prev_node_idx: *tnidx,
						};
						new_tail_nodes.push(pn);
					}
					Obj::Boxx | Obj::BoxxInHole => { 
						// What's in our reverse direction? We can pull into Space and Hole (and HumanInHole cos we won't be there), nothing else.
						let bnpt = &pt.add(&movedir.to_vector().mul(-1));
						match self.level.get_obj_at_pt(bnpt) {
							Obj::Space | Obj::Hole | Obj::Human | Obj::HumanInHole => { 
								// yep, its a keypull, save key move.. 
								let km = KeyMove {
									pn: tnode.clone(),
									move_dir: movedir.clone().reverse(),
								};
								self.key_moves.push(km);
							},
							_ => {} // can't pull the boxx				
						}
					}
					_ => {} // not a move we can take
				};
			}	
		}

		// append new tail nodes to nodes and tail nodes
		self.tail_nodes.clear();
		for n in new_tail_nodes {
			self.nodes.push(n);
			self.tail_nodes.push(self.nodes.len()-1);
		}
	}
	pub fn double_boxx_situation(&self, human_pos: Vector, pushdir: Move) -> bool {
		// checks for a situation where we would be pushing the boxx next to another boxx against a wall and getting ourselves stuck
		//         a = anything, h = human, pushdir = down, * = boxx, ## = wall, ' ' = space, only need row 1 or 3 not both
		//  aa*#	match1
		//  H* #	match0
		//  aa*#	match1
		// test 1 (horizontal in pushdir direction): [ Obj::Boxx, Obj::Space, Obj::Wall ]
		// test 2: above OR below (either or both, +1 in pushdir direction than above): [ Obj::Boxx, Obj::Wall ]
		// 
		// this method improves solution time by about 4-5%

		let match0 = vec![Obj::Boxx, Obj::Space, Obj::Wall];
		let match1 = vec! [ vec![Obj::Boxx, Obj::Wall],
						    vec![Obj::BoxxInHole, Obj::Wall] ];

		let vecs0 = vec![human_pos.add(&pushdir.to_vector()),
						 human_pos.add(&pushdir.to_vector().mul(2)),
						 human_pos.add(&pushdir.to_vector().mul(3))];
		let line0: Vec::<Obj> = vecs0.into_iter().map(|v| self.level.get_obj_at_pt(&v)).collect();
		let vecs1 = vec![human_pos.add(&pushdir.to_vector().mul(2)).add(&pushdir.to_vector().rotl()),
						 human_pos.add(&pushdir.to_vector().mul(3)).add(&pushdir.to_vector().rotl())];
		let vecs2 = vec![human_pos.add(&pushdir.to_vector().mul(2)).add(&pushdir.to_vector().rotr()),
						 human_pos.add(&pushdir.to_vector().mul(3)).add(&pushdir.to_vector().rotr())];
		let line1: Vec::<Obj> = vecs1.into_iter().map(|v| self.level.get_obj_at_pt(&v)).collect();
		let line2: Vec::<Obj> = vecs2.into_iter().map(|v| self.level.get_obj_at_pt(&v)).collect();

		line0 == match0 && ( line1 == match1[0] || line2 == match1[0] )
		// line0 == match0 && ( line1 == match1[0] || line2 == match1[0] || line1 == match1[1] || line2 == match1[1] ) // slows us down by 2.5%
		// line0 == match0 && ( contains_only(&match1, &line1) || contains_only(&match1, &line2) ) // too slow
	}
	pub fn is_map_complete(&self) -> bool { 					// lets us know if there are no more tail nodes (map is complete)
		self.tail_nodes.len() == 0	
	}
	pub fn apply_key_push(level: &SpLevel, km: &KeyMove) -> SpLevel {
		let mut level = level.clone();
		
		// remove old human
		let idx = level.human_pos.to_index(&level.w);
		let human_obj = level.get_obj_at_idx(idx);
		let new_obj = match human_obj {
			Obj::Human => { Obj::Space },
			Obj::HumanInHole => { Obj::Hole },
			_ => { panic!("Human not in tracked location!"); }
		};
		level.set_obj_at_idx(idx, new_obj);
		
		// new human point
		let np = km.pn.pt.add(&km.move_dir.to_vector());
		let idx = np.to_index(&level.w);	
		
		// check destination point
		let obj = level.get_obj_at_idx(idx);
		let new_obj = match obj {
			Obj::Space => { panic!("found space, expecting boxx"); },
			Obj::Hole  => { panic!("found hole, expecting boxx"); },
			Obj::Boxx | Obj::BoxxInHole => {  
				// Move boxx in to next square
				let boxx_pt = &np.add(&km.move_dir.to_vector());
				let i = boxx_pt.to_index(&level.w);
				let o = level.get_obj_at_idx(i);
				if o == Obj::Hole {
					level.set_obj_at_idx(i, Obj::BoxxInHole);
				} else if o == Obj::Space {
					level.set_obj_at_idx(i, Obj::Boxx);
				} else {
					panic!("trying to push boxx into unexpected obj");
				}
			
				// We pushed the boxx
				if obj == Obj::BoxxInHole {
					Obj::HumanInHole
				} else {
					Obj::Human
				}
			},
			_ => { panic!("Human not allowed there!"); }
		};

		// place human
		level.set_obj_at_idx(idx, new_obj);	
		level.human_pos = np;

		level
	}
	pub fn apply_key_pull(level: &SpLevel, km: &KeyMove) -> SpLevel {
		let mut level = level.clone();
		//println!("keymove: pn.pt: {}, move_dir: {}",km.pn.pt.to_string(),km.move_dir.to_string());
		
		// remove old human
		let human_obj = level.get_obj_at_pt(&level.human_pos);
		let human_pos = level.human_pos;
		let new_obj = match human_obj {
			Obj::Human       => { Obj::Space },
			Obj::HumanInHole => { Obj::Hole },
			_ => { panic!("Human not in tracked location!"); }
		};
		level.set_obj_at_pt(&human_pos, new_obj);
		
		// remove old boxx
		let pull_from_pt = km.pn.pt.add(&km.move_dir.reverse().to_vector());
		let pull_obj = level.get_obj_at_pt(&pull_from_pt);
		//println!("pull: pt: {}, obj: {}",pull_pt.to_string(),pull_obj.to_char().to_string());
		let new_obj = match pull_obj {
			Obj::Boxx       => { Obj::Space },
			Obj::BoxxInHole => { Obj::Hole },
			_ => { panic!("Key pull doesn't seem to be moving a boxx!"); }
		};
		level.set_obj_at_pt(&pull_from_pt, new_obj);

		// place new boxx
		let pull_to_pt = &km.pn.pt;
		let pull_to_obj = level.get_obj_at_pt(pull_to_pt);
		let new_obj = match pull_to_obj {
			Obj::Space		=> { Obj::Boxx },
			Obj::Hole		=> { Obj::BoxxInHole },
			_ => { panic!("Key pull seems to be moving boxx into something weird!"); }
		};
		level.set_obj_at_pt(pull_to_pt, new_obj);
	
		// new human point
		let np = km.pn.pt.add(&km.move_dir.to_vector());
		let obj = level.get_obj_at_pt(&np);
		let new_obj = match obj {
			Obj::Space => { Obj::Human },
			Obj::Hole  => { Obj::HumanInHole },
			_ => { panic!("Expected space or hole to move human into"); }
		};
		level.set_obj_at_pt(&np, new_obj);
		level.human_pos = np;

		level
	}
	pub fn new_by_applying_key_push(&self, km: &KeyMove) -> PathNodeMap { 	// after we complete a map, we need to take a key move and start again
		let level = PathNodeMap::apply_key_push(&self.level, km);
		
		let initial_pn = PathNode {
			pt: level.human_pos.clone(),
			steps: km.pn.steps + 1,
			move_taken: None,
			prev_node_idx: 0,
		};
		let mut tail_nodes = Vec::<usize>::with_capacity(32);
		tail_nodes.push(0);
		let mut path = self.path.clone();
		path.append(&mut self.backtrace_moves(&km.pn));
		path.push(km.move_dir);
		PathNodeMap {
			base_level: self.base_level.clone(),
			level: level,
			nodes: vec![initial_pn],
			tail_nodes: tail_nodes,
			key_moves: Vec::<KeyMove>::with_capacity(8),
			path: path,
			contender_flag: false,
			depth: 0,
		}
	}
	pub fn new_by_applying_key_pull(&self, km: &KeyMove) -> PathNodeMap { 	// after we complete a map, we need to take a key move and start again
		let level = PathNodeMap::apply_key_pull(&self.level, km); // this only modifies the level
		
		let initial_pn = PathNode {
			pt: level.human_pos.clone(),
			steps: km.pn.steps + 1,
			move_taken: None,
			prev_node_idx: 0,
		};
		let mut tail_nodes = Vec::<usize>::with_capacity(32);
		tail_nodes.push(0);
		
		let mut path = self.path.clone();
		path.append(&mut self.backtrace_moves(&km.pn));
		path.push(km.move_dir);

		PathNodeMap {
			base_level: self.base_level.clone(),
			level: level,
			nodes: vec![initial_pn],
			tail_nodes: tail_nodes,
			key_moves: Vec::<KeyMove>::with_capacity(8),
			path: path,
			contender_flag: false,
			depth: 0,
		}
	}
	pub fn apply_key_pushes(&self) -> Vec<PathNodeMap> {
		let mut nmaps = Vec::<PathNodeMap>::with_capacity(8);
		for km in &self.key_moves {	
			nmaps.push(self.new_by_applying_key_push(&km));
		}
		nmaps
	}
	pub fn apply_key_pulls(&self) -> Vec<PathNodeMap> {
		let mut nmaps = Vec::<PathNodeMap>::with_capacity(8);
		for km in &self.key_moves {	
			nmaps.push(self.new_by_applying_key_pull(&km));
		}
		nmaps
	}
	pub fn is_level_complete(&self) -> bool {				// after we take a key move, we need to check if we've won the game
		self.level.have_win_condition()	
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
				pnr = &self.nodes[pnr.prev_node_idx];
			} else {
				break;
			}
		}
		path.reverse();
		path
	}
}

