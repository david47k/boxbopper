// PathNodeMap and family
// Used for creating and solving levels

use boxbopperbase::{Obj};
use boxbopperbase::level::{Level,SpLevel2};
use boxbopperbase::vector::{Vector,VectorSm,Move,ALLMOVES};

#[derive(Clone,Copy)]
pub struct PathNode {
	pt: VectorSm,
	pub steps: u32,
	move_taken: Option<Move>, // what move we took to get here, used to determine movelist when solution found
	prev_node_idx: usize,
}

#[derive(Clone,Copy)]
pub struct KeyMove {
	pn: PathNode,		// where human is just before pushing boxx
	move_dir: Move,		// direction to move to push boxx (or direction we are pulling box in)
}


pub fn convert_pts(old: &Vec::<Vector>, w: u16, h: u16) -> Vec<Vector> {
	let mut n = Vec::<Vector>::new();
	for v in old {
		let nv = v.add(&Vector(-1,-1));
		if nv.0 >= 0 && nv.0 < (w) as i32 && nv.1 >= 0 && nv.1 < (h) as i32 {
			n.push(nv);
			print!("pt ({},{}) ",nv.0,nv.1)
		}
	}
	n
}


pub fn convert_to_v8(src: &Vec::<Vector>) -> Vec::<VectorSm> {
    let mut dest = Vec::<VectorSm>::with_capacity(src.len());
    for v in src {
        dest.push(VectorSm::from(v));
    }
    dest
}


#[derive(Clone)]
pub struct PathNodeMap {
	pub level: SpLevel2,
	pub nodes: Vec::<PathNode>,
	tail_nodes: Vec::<usize>,
	key_moves: Vec::<KeyMove>,
	pub path: Vec::<Move>,
	pub depth: u32,
	pub contender_flag: bool,
}

impl PathNodeMap {
	pub fn new_from_level(level: &Level) -> PathNodeMap {				// start the game this way
		let mut map = PathNodeMap {
			level: SpLevel2 { human_pos: VectorSm::from(&level.human_pos), boxx_pts: convert_to_v8(level.get_boxx_pts()) },
			nodes: Vec::<PathNode>::with_capacity(64),
			tail_nodes: Vec::<usize>::with_capacity(32),
			key_moves: Vec::<KeyMove>::with_capacity(16),
			path: Vec::<Move>::with_capacity(64),
			contender_flag: false,
			depth: 0,
		};
		map.nodes.push(PathNode {
			pt: VectorSm::from(&level.human_pos),
			steps: 0,
			move_taken: None,
			prev_node_idx: 0,
		});
		map.tail_nodes.push(0);
		map
	}
	pub fn complete_map_solve(&self, base_level: &Level) -> PathNodeMap {
		let mut map = self.clone();
		while map.tail_nodes.len() != 0 {			// check if map is complete
			map.step_solve(base_level);
		}
		map
	}
	pub fn complete_map_unsolve(&self, base_level: &Level) -> PathNodeMap {
		let mut map = self.clone();
		while map.tail_nodes.len() != 0 {			// check if map is complete
			map.step_unsolve(base_level);
		}		
		map
	}
	pub fn step_solve(&mut self, base_level: &Level) { 										// steps tail nodes forwards one		
		let mut new_tail_nodes = Vec::<PathNode>::with_capacity(32);	// somewhere to store new tail nodes
        
		for tnidx in self.tail_nodes.iter() {							// for each tail node
			let tnode = &self.nodes[*tnidx];
			for movedir in ALLMOVES.iter() {							// for each possible move
				let pt = tnode.pt;									
                let npt = pt.addv(&movedir.to_vector());					// what is in this direction? let's find out	
                if !base_level.vector_in_bounds8(&npt) {
                    continue;
                }
                if self.level.boxx_pts.contains(&npt) {
                    // What's past the boxx? We can push into Space and Hole.
                    let bnpt = &pt.add(&VectorSm::from(&movedir.to_vector().double()));
                    if !self.level.boxx_pts.contains(bnpt) && base_level.vector_in_bounds8(bnpt) && !base_level.in_wall_pts8(bnpt) {
                        // yep, its a keymove, save key move.. but before we do, make sure it isn't a double boxx situation or in our noboxx list
                        if !base_level.in_noboxx_pts8(bnpt) && !self.double_boxx_situation(pt.intov(),*movedir,base_level) {
                            let km = KeyMove {
                                pn: tnode.clone(),
                                move_dir: *movedir,
                            };
                            self.key_moves.push(km);
                        }
                    }
                } else if !base_level.in_wall_pts8(&npt) {
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
			}	
		}

		// append new tail nodes to nodes and tail nodes
		self.tail_nodes.clear();
		for n in new_tail_nodes {
			self.nodes.push(n);
			self.tail_nodes.push(self.nodes.len()-1);
        }
	}
	pub fn step_unsolve(&mut self, base_level: &Level) { 									// steps tail nodes forwards one		
		let mut new_tail_nodes = Vec::<PathNode>::with_capacity(32);	// somewhere to store new tail nodes
		
		for tnidx in self.tail_nodes.iter() {							// for each tail node
			let tnode = &self.nodes[*tnidx];
			for movedir in ALLMOVES.iter() {							// for each possible move
				let pt = tnode.pt;									
                let npt = pt.addv(&movedir.to_vector());					// what is in this direction? let's find out	
				match self.level.get_obj_at_pt_checked(&npt, base_level) {
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
						// What's in our reverse direction? We can pull into Space and Hole.
                        let bnpt = &pt.addv(&movedir.to_vector().mul(-1));
						match self.level.get_obj_at_pt_checked(bnpt, base_level) {
							Obj::Space | Obj::Hole => { 
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
	pub fn double_boxx_situation(&self, human_pos: Vector, pushdir: Move, base_level: &Level) -> bool {
		// checks for a situation where we would be pushing the boxx next to another boxx against a wall and getting ourselves stuck
		//         a = anything, h = human, pushdir = right, * = boxx, # = wall, ' ' = space, only need row 1 or 3 not both
		//  aa*#	match1
		//  h* #	match0
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
		let line0: Vec::<Obj> = vecs0.into_iter().map(|v| VectorSm::from(&v)).map(|v| self.level.get_obj_at_pt_checked(&v, base_level)).collect();
		let vecs1 = vec![human_pos.add(&pushdir.to_vector().mul(2)).add(&pushdir.to_vector().rotl()),
						 human_pos.add(&pushdir.to_vector().mul(3)).add(&pushdir.to_vector().rotl())];
		let vecs2 = vec![human_pos.add(&pushdir.to_vector().mul(2)).add(&pushdir.to_vector().rotr()),
						 human_pos.add(&pushdir.to_vector().mul(3)).add(&pushdir.to_vector().rotr())];
		let line1: Vec::<Obj> = vecs1.into_iter().map(|v| VectorSm::from(&v)).map(|v| self.level.get_obj_at_pt_checked(&v, base_level)).collect();
		let line2: Vec::<Obj> = vecs2.into_iter().map(|v| VectorSm::from(&v)).map(|v| self.level.get_obj_at_pt_checked(&v, base_level)).collect();

		line0 == match0 && ( line1 == match1[0] || line2 == match1[0] )
		// line0 == match0 && ( line1 == match1[0] || line2 == match1[0] || line1 == match1[1] || line2 == match1[1] ) // slows us down by 2.5%
		// line0 == match0 && ( contains_only(&match1, &line1) || contains_only(&match1, &line2) ) // too slow
	}
	pub fn apply_key_push(level: &SpLevel2, km: &KeyMove) -> SpLevel2 {
		let mut level = level.clone();
			
		// new human point
        let np = km.pn.pt.addv(&km.move_dir.to_vector());
        
        // check destination point
        if level.boxx_pts.contains(&np) {
            // move box in to next square
            let boxx_ref = level.boxx_pts.iter_mut().find(|&& mut p| p==np).unwrap();
            *boxx_ref = VectorSm::from(&boxx_ref.intov().add(&km.move_dir.to_vector()));
            //level.boxx_pts[boxx_idx].add(&km.move_dir.to_vector());
        }

		level.human_pos = np;	// place human

		level
	}
	pub fn apply_key_pull(level: &SpLevel2, km: &KeyMove) -> SpLevel2 {
		let mut level = level.clone();
                
        // move (pull) boxx
        let boxx_pt_a = km.pn.pt.addv(&km.move_dir.reverse().to_vector());
        let boxx_ref = level.boxx_pts.iter_mut().find(|&& mut p| p==boxx_pt_a).unwrap();
        *boxx_ref =km.pn.pt;
	
		// new human point
        let np = km.pn.pt.addv(&km.move_dir.to_vector());
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
	pub fn is_level_complete(&self, base_level: &Level) -> bool {				// after we take a key move, we need to check if we've won the game
		self.level.have_win_condition(base_level)	
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

