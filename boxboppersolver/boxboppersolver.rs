
// Box Bopper: Sokoban clone in rust

//use std::io;
//use std::io::{BufReader,BufRead};
//use std::fs::File;
use std::cmp::Ordering;
use rayon::prelude::*;

use std::sync::{Arc,Mutex};
use std::sync::atomic::Ordering as AtomicOrdering;
use std::sync::atomic::*;

use boxbopperbase::{Obj,moves_to_string};
use boxbopperbase::level::{load_level,Level,SpLevel};
use boxbopperbase::vector::{Vector,Move};

const USE_MULTITHREAD: bool = true;

#[derive(Clone,Copy)]
struct PathNode {
	pt: Vector,
	steps: u32,
	move_taken: Option<Move>, // what move we took to get here, used to determine movelist when solution found
	prev_node_idx: usize,
}

#[derive(Clone,Copy)]
struct KeyMove {
	pn: PathNode,		// where human is just before pushing boxx
	push_dir: Move,		// direction to move to push boxx
}

#[derive(Clone)]
struct PathNodeMap {
	base_level: &Level,
	level: SpLevel,
	nodes: Vec::<PathNode>,
	tail_nodes: Vec::<usize>,
	key_moves: Vec::<KeyMove>,
	moves_taken: Vec::<Move>,
}

const AMOVES: [Move;4] = [ Move::Right, Move::Up, Move::Left, Move::Down ];

impl PathNodeMap {
	pub fn new_from_level(level: &Level) -> PathNodeMap {				// start the game this way
		let mut map = PathNodeMap {
			base_level: level,
			level: SpLevel::new_from_level(level),
			nodes: Vec::<PathNode>::with_capacity(64),
			tail_nodes: Vec::<usize>::with_capacity(32),
			key_moves: Vec::<KeyMove>::with_capacity(16),
			moves_taken: Vec::<Move>::with_capacity(64),
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
	pub fn clone_and_complete(&self) -> PathNodeMap {
		let mut map = self.clone();
		map.do_map();
		map
	}
	pub fn do_map(&mut self) {
		while !self.is_map_complete() {
			self.step();
		}
	}
	pub fn step(&mut self) { 												// steps tail nodes forwards one		
		let mut new_tail_nodes = Vec::<PathNode>::with_capacity(32);	// somewhere to store new tail nodes
		
		for tnidx in self.tail_nodes.iter() {									// for each tail node
			let tnode = &self.nodes[*tnidx];
			for movedir in AMOVES.iter() {							// for each possible move
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
						let bnpt = &pt.add(&movedir.to_vector().double());
						match self.level.get_obj_at_pt(bnpt) {
							Obj::Space | Obj::Hole => { 
								// yep, its a keymove, save key move
								if !self.base_level.in_noboxx_pts(*bnpt) {
									let km = KeyMove {
										pn: tnode.clone(),
										push_dir: *movedir,
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
	pub fn get_key_moves(&self) -> Vec<PathNodeMap> {
		let mut nmaps = Vec::<PathNodeMap>::with_capacity(8);
		for km in &self.key_moves {	
			nmaps.push(self.new_by_applying_key_move(&km));
		}
		nmaps
	}
	pub fn is_level_complete(&self) -> bool {				// after we take a key move, we need to check if we've won the game
		self.level.have_win_condition()	
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
	pub fn method_two(map: &mut PathNodeMap, depth: u32, max_steps: &mut u32) {
		if depth == 2 {
			println!("{}...", moves_to_string(&map.moves_taken));
		}
		while !map.is_map_complete() {
			map.step();
		}
		// sort by which is shorter
		map.key_moves.sort_unstable_by(|a,b| if a.pn.steps<b.pn.steps { Ordering::Less }
			else if a.pn.steps==b.pn.steps { Ordering::Equal }
			else { Ordering::Greater }
		);
		for km in map.key_moves.iter() {
			let mut nmap = map.new_by_applying_key_move(km);
			if nmap.is_level_complete() {
				if nmap.nodes[0].steps < *max_steps {
					*max_steps = nmap.nodes[0].steps;
					println!("----- Level complete! -----");
					// Track moves we took to get here!
					println!("Solution in {} moves",nmap.nodes[0].steps);
					println!("Solution: {}", moves_to_string(&nmap.moves_taken));
				}
			} else if nmap.nodes[0].steps < *max_steps {
				PathNodeMap::method_two(&mut nmap,depth+1,max_steps);
			}
		}
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
	let max_steps = Arc::new(AtomicU32::new(1000_u32));
	let mut method: u32 = 1;
	let args: Vec::<String> = std::env::args().collect();
	for arg in args {
		count += 1;
		if count == 2 {
			filename = arg;
		} else if count == 3 {
			let n_maxsteps: u32 = arg.parse::<u32>().unwrap() + 1;
			max_steps.store(n_maxsteps, AtomicOrdering::SeqCst);
		} else if count == 4 {
			method = arg.parse().unwrap();
		}
	}
	
	// load level
	let base_level = load_level(&filename).expect("Unable to load level file");
	let base_map = PathNodeMap::new_from_level(&base_level);
	display_level(&base_level);
	base_map.display_state();

	let mut maps = Vec::<PathNodeMap>::new();
	maps.push(base_map);
	
	let have_solution = Arc::new(AtomicBool::new(false));
	let mut count: u32 = 0;
	let best_solution_str = Arc::new(Mutex::new(String::new()));

	// method 1
	if method == 1 {
		while count < max_steps.load(AtomicOrdering::SeqCst) {	// stop it running forever, it's unlikely to actually get that high
			count += 1;
			println!("----- depth {} -----", count);
			println!("completing {:>7} maps", maps.len());

			let mut nmaps: Vec<PathNodeMap>;
			if USE_MULTITHREAD {
				nmaps = maps.par_iter().map(|m| m.clone_and_complete() ).collect();
			} else {
				nmaps = maps.iter().map(|m| m.clone_and_complete() ).collect();
			}

			maps.clear();
			maps.append(&mut nmaps);
			println!("applying key moves");
			let mut nextmaps = Vec::<PathNodeMap>::new();
			//println!("Number of maps: {}", maps.len());

			if USE_MULTITHREAD {
				// apply key moves
				println!("flatmap..");
				nextmaps = maps.iter().flat_map(|map| map.get_key_moves()).collect();
				
				println!("solution check...");
				// check for level complete / having solution
				nextmaps.par_iter().filter(|m| m.is_level_complete()).for_each(|m| {
					if m.nodes[0].steps < max_steps.load(AtomicOrdering::SeqCst) {
						have_solution.store(true, AtomicOrdering::SeqCst);
						max_steps.store(m.nodes[0].steps, AtomicOrdering::SeqCst);
						println!("----- Level complete! -----");
						let mut solstr = best_solution_str.lock().unwrap();
						*solstr = format!("Solution in {} moves: {}",m.nodes[0].steps,moves_to_string(&m.moves_taken));
						println!("{}",solstr);
					}
				});

				// filter in the ones that haven't reached max steps
				nextmaps = nextmaps.par_iter().filter(|m| m.nodes[0].steps < max_steps.load(AtomicOrdering::SeqCst)).cloned().collect();
			} else {
				for map in maps.iter_mut() {
					for km in map.key_moves.iter() {
						let nmap = map.new_by_applying_key_move(km);
						if nmap.is_level_complete() {
							if !(have_solution.load(AtomicOrdering::SeqCst)) || nmap.nodes[0].steps < max_steps.load(AtomicOrdering::SeqCst) {
								have_solution.store(true, AtomicOrdering::SeqCst);
								max_steps.store(nmap.nodes[0].steps, AtomicOrdering::SeqCst);
								println!("----- Level complete! -----");
								let mut solstr = best_solution_str.lock().unwrap();
								*solstr = format!("Solution in {} moves: {}",nmap.nodes[0].steps,moves_to_string(&nmap.moves_taken));
								println!("{}",solstr);
							}
						} else if !(have_solution.load(AtomicOrdering::SeqCst)) && nmap.nodes[0].steps < max_steps.load(AtomicOrdering::SeqCst) {
							nextmaps.push(nmap);
						}
					}
				}
			}

			if count >= 3 {
				println!("deduping: before {:>7}", nextmaps.len());
				println!("sorting...");
				nextmaps.sort_unstable_by(|a,b| {
					let ord = a.level.partial_cmp(&b.level).unwrap();
					if ord == Ordering::Equal {
						if a.nodes[0].steps < b.nodes[0].steps {
							return Ordering::Less;
						}
						if a.nodes[0].steps > b.nodes[0].steps {
							return Ordering::Greater;
						}
					}
					ord
				});
				println!("deduping...");
				nextmaps.dedup_by(|a,b| a.level.eq_data(&b.level)); // in theory it keeps the first (smallest steps)
				println!("deduping: after  {:>7}", nextmaps.len());
			} 

			maps.clear();
			maps.append(&mut nextmaps);
			if maps.len() == 0 {
				println!("No more maps to check");
				break;
			}
		}
	} else if method == 2 {
		PathNodeMap::method_two(&mut maps[0], 0, &mut (max_steps.load(AtomicOrdering::SeqCst)));
	}

	if have_solution.load(AtomicOrdering::SeqCst) {
		println!("----- Best solution -----");
		let solstr = best_solution_str.lock().unwrap();
		println!("{}",solstr);//load(AtomicOrdering::SeqCst));
	} else {
		println!("----- No solution found -----");
		println!("Max steps was {}",max_steps.load(AtomicOrdering::SeqCst)-1);
		
	}

	return Ok(());
}

