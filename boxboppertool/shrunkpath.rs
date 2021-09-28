// Box Bopper: Sokoban-like game
// Copyright David Atkinson 2020-2021
//
// shrunkpath.rs: store a path as a (smaller) list of moves

use boxbopperbase::stackstack::{StackStack};
use boxbopperbase::vector::{Move};

// ShrunkPath stores the path string (UDLRLRLR etc.) but with each direction stored as only 2 bits
// It uses StackStack, which limits how long a path can be
// Using PathTrait allows us to swap out the underlying path storage method, more easily, if we are experimenting with different ways
// of storing the path

pub trait PathTrait {
	fn new() -> Self;
	fn clear(&mut self);
	fn len(&self) -> u16;
	fn from_path(path: &Vec::<Move>) -> Self;
	fn push(&mut self, move1: &Move);
	fn push_u8(&mut self, move1: u8);
	fn append_path(&mut self, path: &Vec::<Move>);
	fn to_string(&self) -> String;	
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
	fn clear(&mut self) {
		self.count = 0;
	}
	fn len(&self) -> u16 {
		self.count
	}
	fn from_path(path: &Vec::<Move>) -> Self {
		let mut data = StackStack::<u64>::new();
		let mut x: u64 = 0;
		for i in 0..path.len() {
			if i % 32 == 0 && i != 0 {
				data.push(x);
				x=0;
			}
			x |= (path[i] as u64) << (2*(i%32));
		}
		if path.len() > 0 { data.push(x); }

		Self {
			count: path.len() as u16,
			data: data,
		}
	}
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
	fn clear(&mut self) {
		self.count = 0;
	}
	fn len(&self) -> u16 {
		self.count
	}
	fn from_path(path: &Vec::<Move>) -> Self {
		let mut data = StackStack::<u128>::new();
		let mut x: u128 = 0;
		for i in 0..path.len() {
			if i % 64 == 0 && i != 0 {
				data.push(x);
				x=0;
			}
			x |= (path[i] as u128) << (2*(i%64));
		}
		if path.len() > 0 { data.push(x); }

		let rval = Self {
			count: path.len() as u16,
			data: data,
		};

		rval
	}
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
}