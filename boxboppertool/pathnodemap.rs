// boxboppertool Copyright 2020-2021 David Atkinson
//
// pathnodemap.rs: PathNode, PathMap, PathNodeMap and family
// Used for creating and solving levels

use boxbopperbase::{Obj};
use boxbopperbase::level::{Level,SpLevel,CmpData};
use boxbopperbase::vector::{Vector,Move,ALLMOVES,ShrunkPath,SuperShrunkPath,PathTrait};
use boxbopperbase::stackstack::{StackStack16x64,StackStack8x64};

#[derive(Clone,Copy)]
pub struct PathNode {
	pt: Vector,
	prev_node_idx: u16,
	move_taken: Option<Move>, // what move we took to get here, used to determine movelist when solution found
}

#[derive(Clone,Copy)]
pub struct KeyMove {
	pni: u16,			// where human is just before pushing boxx - pathnode index
	move_dir: Move,		// direction to move to push boxx (or direction we are pulling box in)
}

#[derive(Clone)]
pub struct PathNodeMap {
	pub nodes: Vec::<PathNode>,
	pub key_moves: Vec::<KeyMove>,	
}

#[derive(Clone)]
pub struct PathMap {
	pub level: SpLevel,
	pub path: ShrunkPath,
	pub depth: u16,
	pub flag: bool,
}

impl PathMap {
	pub fn new() -> PathMap {
		PathMap {
			level: SpLevel {
				w: 0,
				h: 0,
				cmp_data: CmpData::new(),
			},
			path: PathTrait::new(),
			depth: 0,
			flag: false,
		}
	}
	pub fn new_from_level(level: &Level) -> PathMap {
		PathMap {
			level: SpLevel::from_level(level),
			path: PathTrait::new(),
			depth: 0,
			flag: false,
		}
	}
	pub fn to_pnm(&self) -> PathNodeMap {		// this one clones across our data
		let initial_pn = PathNode {
			pt: Vector(self.level.cmp_data.human_x as i32, self.level.cmp_data.human_y as i32),
			move_taken: None,
			prev_node_idx: 0,
		};
		let mut nodes = Vec::<PathNode>::with_capacity(256/(std::mem::size_of::<PathNode>()));
		nodes.push(initial_pn);
		PathNodeMap {
			nodes: nodes,
			key_moves: Vec::<KeyMove>::with_capacity(128/(std::mem::size_of::<KeyMove>())),
		}
	}
	pub fn complete_solve_2(&self, base_level: &Level, maps_out: &mut Vec::<PathMap>) {		
		let initial_pn = PathNode {
			pt: Vector(self.level.cmp_data.human_x as i32, self.level.cmp_data.human_y as i32),
			move_taken: None,
			prev_node_idx: 0,
		};
		let mut nodes = Vec::<PathNode>::with_capacity(256/(std::mem::size_of::<PathNode>()));
		nodes.push(initial_pn);

		let mut tail_nodes = StackStack16x64::new(); 
		let mut new_tail_nodes = StackStack16x64::new(); 	// somewhere to store new tail nodes
		tail_nodes.push(0);
		while tail_nodes.len() != 0 {					// check if map is complete
			for idx in 0..tail_nodes.len() {							// for each tail node
				let tnidx = tail_nodes.stack[idx]; 
				let tnode = nodes[tnidx as usize];
				let pt = tnode.pt;									
				'loop_moves: for movedir in ALLMOVES.iter() {			// for each possible move
					let npt = pt.add_dir(&movedir);						// what is in this direction? let's find out
					if !base_level.vector_in_bounds(&npt) { continue; }
					if self.level.is_boxx_at_pt(&npt) {
						// What's past the boxx? We can push into Space and Hole.
						let bnpt = pt.add_dir2(&movedir);
						let nobj = self.level.get_obj_at_pt_nohuman_checked(&bnpt, base_level);
						if nobj == Obj::Space || nobj == Obj::Hole {
							// yep, its a keymove, save key move.. but before we do, make sure it isn't a double boxx situation or in our noboxx list
							if !base_level.in_noboxx_pts(&bnpt)  && !self.double_boxx_situation(pt,*movedir,base_level) {
								let km = KeyMove {
									pni: tnidx,
									move_dir: *movedir,
								};
								//pnm.key_moves.push(km);
								maps_out.push(self.apply_key_push_2(&nodes,&km));
							}
						} 
					} else if base_level.get_obj_at_pt(&npt) != Obj::Wall {											
						// first check this point isn't already in our list!!!						
						for n in &nodes {
							if n.pt == npt { continue 'loop_moves; }		// This is a hot spot 9.88%
						}

						// yep, we can move here, make a new tail node
						let pn = PathNode {
							pt: npt.clone(),
							move_taken: Some(*movedir),
							prev_node_idx: tnidx as u16,
						};
						new_tail_nodes.push(nodes.len() as u16);
						nodes.push(pn);
					}
				}	
			}
	
			// move new_tail_nodes to tail_nodes
			tail_nodes.clone_from(&new_tail_nodes);
			new_tail_nodes.clear();
		}
		// pnm -> new_by_applying_key_push(pnm, pm, km)
	}
	pub fn complete_unsolve_2(&self, base_level: &Level, maps_out: &mut Vec::<PathMap>, depth: u16) {
		let initial_pn = PathNode {
			pt: Vector(self.level.cmp_data.human_x as i32, self.level.cmp_data.human_y as i32),
			move_taken: None,
			prev_node_idx: 0,
		};
		let mut nodes = Vec::<PathNode>::with_capacity(256/(std::mem::size_of::<PathNode>()));
		nodes.push(initial_pn);
		
		let mut tail_nodes = StackStack16x64::new(); 
		let mut new_tail_nodes = StackStack16x64::new();
		tail_nodes.push(0);		
		while tail_nodes.len() != 0 {					// check if map is complete
			for idx in 0..tail_nodes.len() {
				let tnidx = tail_nodes.stack[idx];
				let tnode = nodes[tnidx as usize];
				let pt = tnode.pt;									
				'loop_moves: for movedir in ALLMOVES.iter() {							// for each possible move
					let npt = pt.add_dir(&movedir);							// what is in this direction? let's find out	
					if !base_level.vector_in_bounds(&npt) { continue; }
					if self.level.is_boxx_at_pt(&npt) {
						// What's in our reverse direction? We can pull into Space and Hole.
						let bnpt = pt.add_dir(&movedir.reverse());
						let nobj = self.level.get_obj_at_pt_nohuman_checked(&bnpt, base_level);
						if nobj == Obj::Space || nobj == Obj::Hole {
							// yep, its a keypull, save key move.. 
							let km = KeyMove {
								pni: tnidx,
								move_dir: movedir.reverse(),
							};
							//pnm.key_moves.push(km);
							maps_out.push(self.apply_key_pull_2(&nodes,&km, depth));
						}
					} else if base_level.get_obj_at_pt(&npt) != Obj::Wall {
						// first check this point isn't already in our list!!!						
						for n in nodes.iter() {
							if n.pt == npt { continue 'loop_moves; }
						}
						
						// yep, we can move here, make a new tail node
						let pn = PathNode {
							pt: npt.clone(),
							move_taken: Some(*movedir),
							prev_node_idx: tnidx as u16,
						};
						new_tail_nodes.push(nodes.len() as u16);
						nodes.push(pn);
					}
				}	
			}

			// move new_tail_nodes to tail_nodes
			tail_nodes.clone_from(&new_tail_nodes);
			new_tail_nodes.clear();
		}		
	}
	pub fn apply_key_push_2(&self, nodes: &Vec::<PathNode>, km: &KeyMove) -> PathMap { 	// after we complete a map, we need to take a key move and start again	
		let mut map_b = self.clone();
				
		// new human point
		let np = nodes[km.pni as usize].pt.add_dir(&km.move_dir);
		
		// check destination point
		if map_b.level.is_boxx_at_pt(&np) {
			let boxx_pt = np.add_dir(&km.move_dir);
			let is_clear = !map_b.level.is_boxx_at_pt(&boxx_pt);
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
		
		backtrace_moves2(nodes, km.pni as usize, &mut map_b.path);
		map_b.path.push(&km.move_dir);
		
		map_b
	}
	pub fn apply_key_pull_2(&self, nodes: &Vec::<PathNode>, km: &KeyMove, depth: u16) -> PathMap { 	// after we complete a map, we need to take a key move and start again
		let mut map_b = self.clone();
		map_b.depth = depth;
				
		// remove old boxx
		let pull_from_pt = nodes[km.pni as usize].pt.add_dir(&km.move_dir.reverse());
		let is_boxx = map_b.level.is_boxx_at_pt(&pull_from_pt);
		if is_boxx {
			map_b.level.clear_boxx_at_pt(&pull_from_pt);
		} else {
			panic!("Key pull doesn't seem to be moving a boxx!");
		}

		// place new boxx
		let pull_to_pt = nodes[km.pni as usize].pt;
		let is_clear = !map_b.level.is_boxx_at_pt(&pull_to_pt);
		if is_clear {
			map_b.level.set_boxx_at_pt(&pull_to_pt);
		} else {
			panic!("Key pull seems to be moving boxx into something weird!");
		}
		
		// new human point
		let np = nodes[km.pni as usize].pt.add_dir(&km.move_dir);
		map_b.level.set_human_pos(&np);

		backtrace_moves2(nodes, km.pni as usize, &mut map_b.path);
		map_b.path.push(&km.move_dir);

		map_b
	}	
	pub fn new_by_applying_key_push(pnm: &PathNodeMap, pm: &PathMap, km: &KeyMove) -> PathMap { 	// after we complete a map, we need to take a key move and start again	
		let mut map_b = pm.clone();
				
		// new human point
		let np = pnm.nodes[km.pni as usize].pt.add_dir(&km.move_dir);
		
		// check destination point
		if map_b.level.is_boxx_at_pt(&np) {
			let boxx_pt = np.add_dir(&km.move_dir);
			let is_clear = !map_b.level.is_boxx_at_pt(&boxx_pt);
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
		
		pnm.backtrace_moves(km.pni as usize, &mut map_b.path);
		map_b.path.push(&km.move_dir);
		
		map_b
	}
	pub fn new_by_applying_key_pull(pnm: &PathNodeMap, pm: &PathMap, km: &KeyMove, depth: u16) -> PathMap { 	// after we complete a map, we need to take a key move and start again
		let mut map_b = pm.clone();
		map_b.depth = depth;
				
		// remove old boxx
		let pull_from_pt = pnm.nodes[km.pni as usize].pt.add_dir(&km.move_dir.reverse());
		let is_boxx = map_b.level.is_boxx_at_pt(&pull_from_pt);
		if is_boxx {
			map_b.level.clear_boxx_at_pt(&pull_from_pt);
		} else {
			panic!("Key pull doesn't seem to be moving a boxx!");
		}

		// place new boxx
		let pull_to_pt = pnm.nodes[km.pni as usize].pt;
		let is_clear = !map_b.level.is_boxx_at_pt(&pull_to_pt);
		if is_clear {
			map_b.level.set_boxx_at_pt(&pull_to_pt);
		} else {
			panic!("Key pull seems to be moving boxx into something weird!");
		}
		
		// new human point
		let np = pnm.nodes[km.pni as usize].pt.add_dir(&km.move_dir);
		map_b.level.set_human_pos(&np);

		pnm.backtrace_moves(km.pni as usize, &mut map_b.path);
		map_b.path.push(&km.move_dir);

		map_b
	}
	pub fn complete_map_solve(&self, base_level: &Level) -> PathNodeMap {
		let mut pnm = self.to_pnm();					// we want complete_map to clone from self
		let mut tail_nodes = StackStack16x64::new(); 
		let mut new_tail_nodes = StackStack16x64::new(); 	// somewhere to store new tail nodes
		tail_nodes.push(0);
		while tail_nodes.len() != 0 {					// check if map is complete
			for idx in 0..tail_nodes.len() {							// for each tail node
				let tnidx = tail_nodes.stack[idx]; 
				let tnode = pnm.nodes[tnidx as usize];
				let pt = tnode.pt;									
				'loop_moves: for movedir in ALLMOVES.iter() {			// for each possible move
					let npt = pt.add_dir(&movedir);						// what is in this direction? let's find out
					if !base_level.vector_in_bounds(&npt) { continue; }
					if self.level.is_boxx_at_pt(&npt) {
						// What's past the boxx? We can push into Space and Hole.
						let bnpt = pt.add_dir2(&movedir);
						let nobj = self.level.get_obj_at_pt_nohuman_checked(&bnpt, base_level);
						if nobj == Obj::Space || nobj == Obj::Hole {
							// yep, its a keymove, save key move.. but before we do, make sure it isn't a double boxx situation or in our noboxx list
							if !base_level.in_noboxx_pts(&bnpt)  && !self.double_boxx_situation(pt,*movedir,base_level) {
								let km = KeyMove {
									pni: tnidx,
									move_dir: *movedir,
								};
								pnm.key_moves.push(km);
							}
						} 
					} else if base_level.get_obj_at_pt(&npt) != Obj::Wall {											
						// first check this point isn't already in our list!!!						
						for n in &pnm.nodes {
							if n.pt == npt { continue 'loop_moves; }		// This is a hot spot 9.88%
						}

						// yep, we can move here, make a new tail node
						let pn = PathNode {
							pt: npt.clone(),
							move_taken: Some(*movedir),
							prev_node_idx: tnidx as u16,
						};
						new_tail_nodes.push(pnm.nodes.len() as u16);
						pnm.nodes.push(pn);
					}
				}	
			}
	
			// move new_tail_nodes to tail_nodes
			tail_nodes.clone_from(&new_tail_nodes);
			new_tail_nodes.clear();
		}
		pnm
	}
	pub fn complete_map_unsolve(&self, base_level: &Level) -> PathNodeMap {
		let mut pnm = self.to_pnm();					// we want complete_map to clone from self
		let mut tail_nodes = StackStack16x64::new(); 
		let mut new_tail_nodes = StackStack16x64::new();
		tail_nodes.push(0);		
		while tail_nodes.len() != 0 {					// check if map is complete
			for idx in 0..tail_nodes.len() {
				let tnidx = tail_nodes.stack[idx];
				let tnode = pnm.nodes[tnidx as usize];
				let pt = tnode.pt;									
				'loop_moves: for movedir in ALLMOVES.iter() {							// for each possible move
					let npt = pt.add_dir(&movedir);							// what is in this direction? let's find out	
					if !base_level.vector_in_bounds(&npt) { continue; }
					if self.level.is_boxx_at_pt(&npt) {
						// What's in our reverse direction? We can pull into Space and Hole.
						let bnpt = pt.add_dir(&movedir.reverse());
						let nobj = self.level.get_obj_at_pt_nohuman_checked(&bnpt, base_level);
						if nobj == Obj::Space || nobj == Obj::Hole {
							// yep, its a keypull, save key move.. 
							let km = KeyMove {
								pni: tnidx,
								move_dir: movedir.reverse(),
							};
							pnm.key_moves.push(km);
						}
					} else if base_level.get_obj_at_pt(&npt) != Obj::Wall {
						// first check this point isn't already in our list!!!						
						for n in pnm.nodes.iter() {
							if n.pt == npt { continue 'loop_moves; }
						}
						
						// yep, we can move here, make a new tail node
						let pn = PathNode {
							pt: npt.clone(),
							move_taken: Some(*movedir),
							prev_node_idx: tnidx as u16,
						};
						new_tail_nodes.push(pnm.nodes.len() as u16);
						pnm.nodes.push(pn);
					}
				}	
			}

			// move new_tail_nodes to tail_nodes
			tail_nodes.clone_from(&new_tail_nodes);
			new_tail_nodes.clear();
		}		
		pnm
	}
	pub fn double_boxx_situation(&self, human_pos: Vector, pushdir: Move, base_level: &Level) -> bool {
		// checks for a situation where we would be pushing the boxx next to another boxx against a wall and getting ourselves stuck
		//         a = anything, h = human, pushdir = right, * = boxx, # = wall, ' ' = space, only need row 1 or 3 not both
		//  aa*#	matchB                        pp1     pp1a
		//  h* #	matchA         h_pos  hpadd1  hpadd2  hpadd3
		//  aa*#	matchB                        pp2     pp2a
		// test 1 (horizontal in pushdir direction): [ Obj::Boxx, Obj::Space, Obj::Wall ]
		// test 2: above OR below (either or both, +1 in pushdir direction than above): [ Obj::Boxx, Obj::Wall ]
		// 
		// this method improves solution time by about 4-5%

		let pushv = pushdir.to_vector();

		let hpadd1 = human_pos.add(&pushv);
		let hpadd2 = human_pos.add(&pushv.mul(2));

		if !(base_level.vector_in_bounds(&hpadd1) && base_level.vector_in_bounds(&hpadd2)) {
			return false;
		}

		let hpadd3 = human_pos.add(&pushv.mul(3));

		let line0  = [ self.level.get_obj_at_pt_nohuman(&hpadd2, base_level),
					   self.level.get_obj_at_pt_nohuman_checked(&hpadd3, base_level) ];
		
		if line0 == [Obj::Space, Obj::Wall] {
			let pp1 = hpadd2.add(&pushv.rotl());
			if base_level.vector_in_bounds(&pp1) && self.level.is_boxx_at_pt(&pp1) {
				let pp1a = hpadd3.add(&pushv.rotl());
				if base_level.get_obj_at_pt_checked(&pp1a) == Obj::Wall {
					return true;
				}
			}
			let pp2 = hpadd2.add(&pushv.rotr());
			if base_level.vector_in_bounds(&pp2) && self.level.is_boxx_at_pt(&pp2) {
				let pp2a = hpadd3.add(&pushv.rotr());
				if base_level.get_obj_at_pt_checked(&pp2a) == Obj::Wall {
					return true;
				}
			}
		}

		return false;
	}	
}

impl PathNodeMap {
	pub fn apply_key_pushes(&self, base_path_map: &PathMap) -> Vec<PathMap> {			// avoid using these as they are slow
		let mut nmaps = Vec::<PathMap>::with_capacity(self.key_moves.len());
		for km in &self.key_moves {	
			nmaps.push(PathMap::new_by_applying_key_push(self, base_path_map, &km));
		}
		nmaps
	}
	pub fn apply_key_pulls(&self, base_path_map: &PathMap) -> Vec<PathMap> {				// avoid using these as they are slow
		let mut nmaps = Vec::<PathMap>::with_capacity(self.key_moves.len());
		for km in &self.key_moves {	
			nmaps.push(PathMap::new_by_applying_key_pull(self, base_path_map, &km, 0));
		}
		nmaps
	}
	pub fn backtrace_moves(&self, pni: usize, spath: &mut impl PathTrait) {		// 5.5, 2.9
		let mut path = StackStack8x64::new();
		// start at pn and work backwards
		let mut pnr = &self.nodes[pni];
		loop {
			if pnr.move_taken.is_some() {
				path.push(pnr.move_taken.unwrap() as u8);
				if pnr.prev_node_idx == 0 {
					let m = &self.nodes[0].move_taken;
					if m.is_some() { path.push(m.unwrap() as u8); }
					break;
				}
				pnr = &self.nodes[pnr.prev_node_idx as usize];
			} else {
				break;
			}
		}
		
		for i in 1..=path.next {
			let rev = path.next - i;
			spath.push_u8(path.stack[rev]);	//3.88%
		}
	}
}


pub fn backtrace_moves2(nodes: &Vec::<PathNode>, pni: usize, spath: &mut impl PathTrait) {		// 5.5, 2.9
	let mut path = StackStack8x64::new();
	// start at pn and work backwards
	let mut pnr = nodes[pni];
	loop {
		if pnr.move_taken.is_some() {
			path.push(pnr.move_taken.unwrap() as u8);
			if pnr.prev_node_idx == 0 {
				let m = nodes[0].move_taken;
				if m.is_some() { path.push(m.unwrap() as u8); }
				break;
			}
			pnr = nodes[pnr.prev_node_idx as usize];
		} else {
			break;
		}
	}
	
	for i in 1..=path.next {
		let rev = path.next - i;
		spath.push_u8(path.stack[rev]);	//3.88%
	}
}

