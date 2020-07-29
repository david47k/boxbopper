
// Box Bopper: Sokoban clone in rust

//use std::io;
//use std::io::{BufReader,BufRead};
//use std::fs::File;

//mod boxbopperbase;

use boxbopperbase::{Obj,moves_to_string};
use boxbopperbase::level::{load_level,Level};
use boxbopperbase::vector::{Vector,Move,ALLMOVES};

#[derive(Clone,Copy)]
struct PathNode {
	pt: Vector,
	steps: u32,
	move_taken: Option<Move>, // what move we took to get here, used to determine movelist when solution found
	prev_node_idx: usize,
}

struct KeyMove {
	pn: PathNode,		// where human is just before pushing boxx
	push_dir: Move,		// direction to move to push boxx
}

struct PathNodeMap {
	level: Level,
	nodes: Vec::<PathNode>,
	tail_nodes: Vec::<usize>,
	key_moves: Vec::<KeyMove>,
	moves_taken: Vec::<Move>,
}

impl PathNodeMap {
	pub fn new_from_level(level: &Level) -> PathNodeMap {				// start the game this way
		let mut map = PathNodeMap {
			level: level.clone(),
			nodes: Vec::<PathNode>::with_capacity(64),
			tail_nodes: Vec::<usize>::with_capacity(32),
			key_moves: Vec::<KeyMove>::with_capacity(8),
			moves_taken: Vec::<Move>::with_capacity(8),
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
	pub fn step(&mut self) { 												// steps tail nodes forwards one		
		let mut new_tail_nodes = Vec::<PathNode>::with_capacity(32);	// somewhere to store new tail nodes
		
		for tnidx in self.tail_nodes.iter() {									// for each tail node
			let tnode = &self.nodes[*tnidx];
			for movedir in ALLMOVES.iter() {							// for each possible move
				let pt = tnode.pt;									
				let npt = pt.add(&movedir.to_vector());						// what is in this direction? let's find out	
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
					Obj::Boulder | Obj::BoulderInHole => { 
						// What's past the boulder? We can push into Space and Hole, nothing else.
						match self.level.get_obj_at_pt(&pt.add(&movedir.to_vector().double())) {
							Obj::Space | Obj::Hole => { 
								// yep, its a keymove, save key move
								let km = KeyMove {
									pn: tnode.clone(),
									push_dir: *movedir,
								};
								self.key_moves.push(km);
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
	pub fn is_map_complete(&self) -> bool { 					// lets us know if there are no more tail nodes (map is complete)
		self.tail_nodes.len() == 0	
	}
	pub fn apply_key_move(level: &mut Level, km: &KeyMove) {
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
		let np = km.pn.pt.add(&km.push_dir.to_vector());
		let idx = np.to_index(&level.w);	
		
		// check destination point
		let obj = level.get_obj_at_idx(idx);
		let new_obj = match obj {
			Obj::Space => { panic!("found space, expecting boxx"); },
			Obj::Hole  => { panic!("found hole, expecting boxx"); },
			Obj::Boulder | Obj::BoulderInHole => {  
				// Move boulder in to next square
				let boulder_pt = &np.add(&km.push_dir.to_vector());
				let i = boulder_pt.to_index(&level.w);
				let o = level.get_obj_at_idx(i);
				if o == Obj::Hole {
					level.set_obj_at_idx(i, Obj::BoulderInHole);
				} else if o == Obj::Space {
					level.set_obj_at_idx(i, Obj::Boulder);
				} else {
					panic!("trying to push boxx into unexpected obj");
				}
			
				// We pushed the boulder
				if obj == Obj::BoulderInHole {
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
	}
	pub fn new_by_applying_key_move(&self, km: &KeyMove) -> PathNodeMap { 	// after we complete a map, we need to take a key move and start again
		let mut level = self.level.clone();
		PathNodeMap::apply_key_move(&mut level, km);
		
		let initial_pn = PathNode {
			pt: level.human_pos.clone(),
			steps: km.pn.steps + 1,
			move_taken: None,
			prev_node_idx: 0,
		};
		let mut tail_nodes = Vec::<usize>::with_capacity(32);
		tail_nodes.push(0);
		let mut moves_taken = self.moves_taken.clone();
		moves_taken.append(&mut self.backtrace_moves(&km.pn));
		moves_taken.push(km.push_dir);
		PathNodeMap {
			level: level,
			nodes: vec!(initial_pn),
			tail_nodes: tail_nodes,
			key_moves: Vec::<KeyMove>::with_capacity(8),
			moves_taken: moves_taken,
		}
	}
	pub fn is_level_complete(&self) -> bool {				// after we take a key move, we need to check if we've won the game
		self.level.have_win_condition()	
	}
	pub fn get_movelist() -> String {					// when we win the game, we need to see what moves we took to get here
		"movelist: todo".to_string()		
	}
	pub fn display_state(&self) {
		println!("nodes: {}   tail_nodes: {}   key_moves: {}",self.nodes.len(), self.tail_nodes.len(), self.key_moves.len());
		println!("key_moves:");
		for node in self.key_moves.iter() {
			println!("    at {},{} at {} steps in dir {}", node.pn.pt.0, node.pn.pt.1, node.pn.steps, node.push_dir.to_string());
		}
		println!();
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


pub fn display_level(level: &Level) {
	println!("--------------------------------------------------------------------------------");
	println!();
	// print level
	for y in 0..level.h {
		for x in 0..level.w {
			print!("{}",level.get_obj_at_idx(y * level.w + x).to_char());
		}
		println!();
	}
	println!();
}

fn main() -> Result<(),String> {
	let mut filename: String = String::from("levels/level01.txt");
	
	let mut count = 0;
	for env in std::env::args() {
		if count > 0 {
			filename = env;
		}
		count += 1;
	}
	
	// load level
	let base_level = load_level(&filename).expect("Unable to load level file");
	let mut base_map = PathNodeMap::new_from_level(&base_level);
	display_level(&base_level);
	base_map.display_state();

	let mut maps = Vec::<PathNodeMap>::new();
	maps.push(base_map);
	
	let mut count = 0;
	while count < 100 {
		count += 1;
		println!("----- Depth {} loop start -----", count);
		for map in maps.iter_mut() {
			while !map.is_map_complete() {
				map.step();
			}
			//map.display_state();
		}

		println!("----- Depth {} applying key moves -----", count);
		let mut nextmaps = Vec::<PathNodeMap>::new();
		println!("Number of maps: {}", maps.len());
		for map in maps.iter_mut() {
			for km in map.key_moves.iter() {
				let nmap = map.new_by_applying_key_move(km);
				//println!("For km at {},{}, human at {},{}",km.pn.pt.0,km.pn.pt.1,nmap.level.human_pos.0,nmap.level.human_pos.1);
				//println!("  {} steps: {}", nmap.nodes[0].steps, moves_to_string(&nmap.moves_taken));
				if nmap.is_level_complete() {
					println!("----- Level complete! -----");
					// Track moves we took to get here!
					return Ok(());
				}
				nextmaps.push(nmap);
			}
		}

		maps.clear();
		maps = nextmaps;
	}
	
	return Ok(());
}

