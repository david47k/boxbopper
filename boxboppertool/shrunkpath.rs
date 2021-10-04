// Box Bopper: Sokoban-like game
// Copyright David Atkinson 2020-2021
//
// shrunkpath.rs: store a path as a (smaller) list of moves

use std::sync::{Arc,RwLock,Weak};
use boxbopperbase::stackstack::{StackStack};
use boxbopperbase::vector::{Move};

// ShrunkPath stores the path string (UDLRLRLR etc.) but with each direction stored as only 2 bits
// It uses StackStack, which limits how long a path can be
// Using PathTrait allows us to swap out the underlying path storage method, more easily, if we are experimenting with different ways
// of storing the path

pub trait PathTrait {
	fn new() -> Self;
	//fn clear(&mut self);
	fn len(&self) -> u16;
	//fn from_path(path: &Vec::<Move>) -> Self;
	fn push(&mut self, move1: &Move);
	fn push_u8(&mut self, move1: u8);
	fn append_path(&mut self, path: &Vec::<Move>);
	fn to_string(&self) -> String;	
}

#[derive()]
pub struct TreeNode {
	pub value: u8,
	pub length: u16,		// quicker than looking back..?
	pub children: [ Option<TreeNodeRef>; 4 ],
	pub parent: Option<TreeNodeWeak>,
}

#[derive(Clone)]
pub struct TreeNodeRef {
	inner: Arc<RwLock<TreeNode>>,
}

#[derive(Clone)]
pub struct TreeNodeWeak {
	inner: Weak<RwLock<TreeNode>>,
}

impl TreeNodeWeak {
	pub fn upgrade(self) -> TreeNodeRef {
		TreeNodeRef {
			inner: self.inner.upgrade().expect("TreeNodeWeak won't upgrade!")
		}
	}
}

impl TreeNodeRef {
	pub fn val(&self) -> u8 {
		self.inner.read().unwrap().value
	}
	pub fn parent(&self) -> TreeNodeRef {	// return STRONG form of parent
		self.inner.read().unwrap().parent.clone().unwrap().upgrade()
	}
	pub fn len(&self) -> u16 {
		self.inner.read().unwrap().length
	}
	pub fn push(&mut self, u: u8) -> TreeNodeRef {
		// before we add, we have to check if this move has already been added, in which case all we do is return a ref to that child
		let len = self.len() + 1;
		//println!("Strong count {}, adding {} with len {}", Arc::strong_count(&self.inner), u, len);
		let mut inner = self.inner.write().unwrap();
		match &inner.children[u as usize] {
			Some(r) => return r.clone(),
			None => {},
		}
		let n = TreeNodeRef {
			inner: Arc::new(RwLock::new(TreeNode {
				value: u,
				length: len,
				children: [ None, None, None, None ],
				parent: Some(self.clone().downgrade()),
			})),
		};
		inner.children[u as usize] = Some(n.clone());
		n
	}
	pub fn new_root() -> TreeNodeRef {
		TreeNodeRef {
			inner: Arc::new(RwLock::new(TreeNode {
				value: 0,
				length: 0,
				children: [ None, None, None, None ],
				parent: None,
			})),
		}
	}
	pub fn push_path(&mut self, path: &Vec::<Move>) -> TreeNodeRef {
		if path.len() < 1 {
			return self.clone();
		}
		let mut ptr = self.push(path[0] as u8);
		for i in 1..path.len() {
			ptr = ptr.push(path[i] as u8);
		}
		ptr
	}
	pub fn to_path(&self) -> Vec<Move> {
		let mut moves = Vec::<Move>::new();
		let mut nptr = self.clone();
		while nptr.len() > 0 {
			moves.push(Move::from_u8_unchecked(nptr.val()));
			nptr = nptr.parent();
		}
		moves.reverse();
		moves
	}
	pub fn to_string(&self) -> String {
		let moves = self.to_path();
		let mut output_str = "".to_string();
		for m in moves {
			output_str += &m.to_string();
		}
		output_str
	}
	pub fn print_tree_size(&self) {
		// first find root
		let mut nptr = self.clone();
		while nptr.len() > 0 {
			nptr = nptr.parent();
		}
		// then go down all the paths
		let mut max_strong = 0;
		let mut count = 1;
		let mut childs_outer: Vec<TreeNodeRef> = vec![ nptr ];
		let mut childs_inner: Vec<TreeNodeRef> = vec![];
		while childs_outer.len() > 0 {
			let pop = childs_outer.pop().unwrap();
			for c in pop.inner.read().unwrap().children.iter() {
				match c {
					Some(child) => {
						count += 1;
						let sc = Arc::strong_count(&child.inner);
						if sc > max_strong {
							max_strong = sc;
						}
						childs_inner.push(child.clone());
					},
					None => {},
				}
			}
			if childs_outer.len() == 0 {
				childs_outer.append(&mut childs_inner);
			}
		}
		println!("Tree has {} nodes, {} MB, max strong {}", count, count * std::mem::size_of::<TreeNode>() / (1024*1024), max_strong);
	}
	pub fn downgrade(self) -> TreeNodeWeak {
		TreeNodeWeak {
			inner: Arc::downgrade(&self.inner),
		}
	}
} 

#[derive(Clone)]
pub struct ShrunkPath {
	count: u16,
	data: StackStack<u64>,
}

impl PathTrait for ShrunkPath {
	fn new() -> Self {
		Self {
			count: 0,
			data: StackStack::<u64>::new(),
		}
	}
	// fn clear(&mut self) {
	// 	self.count = 0;
	// }
	fn len(&self) -> u16 {
		self.count
	}
	// fn from_path(path: &Vec::<Move>) -> Self {
	// 	let mut data = StackStack::<u64>::new();
	// 	let mut x: u64 = 0;
	// 	for i in 0..path.len() {
	// 		if i % 32 == 0 && i != 0 {
	// 			data.push(x);
	// 			x=0;
	// 		}
	// 		x |= (path[i] as u64) << (2*(i%32));
	// 	}
	// 	if path.len() > 0 { data.push(x); }

	// 	Self {
	// 		count: path.len() as u16,
	// 		data: data,
	// 	}
	// }
	fn push(&mut self, move1: &Move) {
		if self.count%32==0 { 
			// append new block
			self.data.push(*move1 as u64);
		} else {
			// modify existing block
			let idx = self.count as usize/32;
			let mut x = self.data.stack[idx];
			x |= (*move1 as u64) << (2*(self.count%32));
			self.data.stack[idx] = x;
		}
		self.count += 1;		
	}
	fn push_u8(&mut self, move1: u8) {
		let move1 = move1 as u64;
		if self.count%32==0 { 
			// append new block
			self.data.push(move1);
		} else {
			// modify existing block
			let idx = self.count as usize/32;
			let mut x = self.data.stack[idx];
			x |= (move1) << (2*(self.count%32));
			self.data.stack[idx] = x;
		}
		self.count += 1;		
	}
	fn append_path(&mut self, path: &Vec::<Move>) {
		for move1 in path {
			if self.count%32==0 { 
				// append new block
				self.data.push(*move1 as u64);
			} else {
				// modify existing block
				let idx = self.count as usize/32;
				let mut x = self.data.stack[idx];
				x |= (*move1 as u64) << (2*(self.count%32));
				self.data.stack[idx] = x;
			}
			self.count += 1;
		}
	}	
	fn to_string(&self) -> String {
		let path = self.to_path();
		let mut s: String = "".to_string();
		for m in path.iter() {
			s = s + &m.to_string();
		}
		return s;
	}
}

impl ShrunkPath {
	pub fn with_capacity(_c: usize) -> Self {
		Self {
			count: 0,
			data: StackStack::<u64>::new(),
		}
	}
	pub fn to_path(&self) -> Vec::<Move> {
		let mut path = Vec::<Move>::with_capacity(self.count as usize);
		for i in 0..self.count as usize {
			let block = self.data.stack[i/32];
			let shr = block >> (2*(i%32));
			path.push( Move::from_u64_unchecked( shr & 0x03 ) );
		}
		path
	}
	pub fn get_u(&self, i: usize) -> u64 {
		if i >= self.count as usize { panic!("ShrunkPath::get index is too high"); }
		return (self.data.stack[i/32] >> (2*(i%32))) & 0x03;
	}
	pub fn set_u(&mut self, i: usize, val: u64) {
		if i >= self.count as usize { panic!("ShrunkPath::set index is too high"); }
		let bidx = 2*(i%32);
		let mask: u64 = ! ( 0x03 << bidx );
		let maskdata = self.data.stack[i/32] & mask;
		let newdata = maskdata | (val << bidx);
		self.data.stack[i/32] = newdata;
	}
	pub fn pop(&mut self) -> Move {
		if self.count == 0 { panic!("stack underflow in ShrunkPath::pop"); }
		self.count -= 1;
		let i = self.count as usize;
		let block = self.data.stack[i/32];
		let shr = block >> (2*(i%32));
		return Move::from_u64_unchecked( shr & 0x03 );
	}
	pub fn reverse(&mut self) {
		if self.count < 2 { return; }
		for i in 0..(self.count as usize / 2) {
			let revi = self.count as usize - i - 1;
			let iv = self.get_u(i);
			let rv = self.get_u(revi);
			self.set_u(i, rv);
			self.set_u(revi, iv);
		}		
	}

}


#[derive(Clone)]
pub struct ShrunkPath128 {    
	count: u16,
	data: StackStack::<u128>,
}

impl PathTrait for ShrunkPath128 {
	fn new() -> Self {
		Self {
			count: 0,
			data: StackStack::<u128>::new(),
		}
	}
	fn len(&self) -> u16 {
		self.count
	}
	// fn from_path(path: &Vec::<Move>) -> Self {
	// 	let mut data = StackStack::<u128>::new();
	// 	let mut x: u128 = 0;
	// 	for i in 0..path.len() {
	// 		if i % 64 == 0 && i != 0 {
	// 			data.push(x);
	// 			x=0;
	// 		}
	// 		x |= (path[i] as u128) << (2*(i%64));
	// 	}
	// 	if path.len() > 0 { data.push(x); }

	// 	let rval = Self {
	// 		count: path.len() as u16,
	// 		data: data,
	// 	};

	// 	rval
	// }
	fn push(&mut self, move1: &Move) {
		if self.count%64==0 { 
			// append new block
			self.data.push(*move1 as u128);
		} else {
			// modify existing block
			let idx = self.count as usize/64;
			let mut x = self.data.stack[idx];
			x |= (*move1 as u128) << (2*(self.count%64));
			self.data.stack[idx] = x;
		}
		self.count += 1;
	}
	fn push_u8(&mut self, move1: u8) {
		let move1 = move1 as u128;
		if self.count%64==0 { 
			// append new block
			self.data.push(move1);
		} else {
			// modify existing block
			let idx = self.count as usize/64;
			let mut x = self.data.stack[idx];
			x |= (move1) << (2*(self.count%64));
			self.data.stack[idx] = x;
		}
		self.count += 1;
	}
	fn append_path(&mut self, path: &Vec::<Move>) {
		for move1 in path {		// not the fastest way but reduces code spaghetti
			self.push(move1);
		}
	}	
	fn to_string(&self) -> String {
		let path = self.to_path();
		let mut s: String = "".to_string();
		for m in path.iter() {
			s = s + &m.to_string();
		}
		return s;
	}
}

impl ShrunkPath128 {
	pub fn to_path(&self) -> Vec::<Move> {
		let mut path = Vec::<Move>::with_capacity(self.len() as usize);
		for i in 0..self.count as usize {
			let block = self.data.stack[i/64];
			let shr = block >> (2*(i%64));
			path.push( Move::from_u128_unchecked( shr & 0x03 ) );
		}
		path
	}
	pub fn clear(&mut self) {
		self.count = 0;
	}
}