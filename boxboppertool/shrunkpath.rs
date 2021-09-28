// Box Bopper: Sokoban-like game
// Copyright David Atkinson 2020-2021
//
// shrunkpath.rs: store a path as a (smaller) list of moves

use boxbopperbase::stackstack::{StackStack8x64,StackStack};
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
	fn append_path_ss8(&mut self, ss: &StackStack8x64);
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
	fn append_path_ss8(&mut self, ss: &StackStack8x64) {
		for i in 0..ss.next {
			self.push(&Move::from_u8_unchecked(ss.stack[i]));
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



// SuperShrunkPath stores part of the path as an index to existing path strings... thus allowing us to shorten the path data due to reuse... it'll use a lot more cpu though, to find matching path strings
// Once we get more than 64 moves, we can store the original moves as a 'prefix' index... 64 moves is 128 bits, plus we get reduction in mem usage as there will be lots of repeats

#[derive(Clone)]
pub struct SuperPrefix {
	data: Vec<u128>,
}

impl SuperPrefix {
	pub fn new() -> Self {
		Self {
			data: Vec::new(),
		}
	}
	pub fn get_by_index(&self, i: u32) -> u128 {
		self.data[i as usize]
	}
	pub fn add_bits_without_searching(&mut self, d: u128) -> u32 {
		let i = self.data.len();
		self.data.insert(i, d);
		return i as u32;
	}
	pub fn add_bits(&mut self, d: u128) -> u32 {
		// basic linear search ugh
		for idx in 0..self.data.len() {
			if self.data[idx] == d {
				return idx as u32;
			}
		}
		self.add_bits_without_searching(d)
	}
	pub fn len(&self) -> usize {
		self.data.len()
	}
}

#[derive(Clone)]
pub struct SuperShrunkPath {    
    compressed_data: Option<u32>,
	count: u16,
	data: StackStack::<u128>,
}

impl PathTrait for SuperShrunkPath {
	fn new() -> Self {
		Self {
			compressed_data: None,
			count: 0,
			data: StackStack::<u128>::new(),
		}
	}
	fn clear(&mut self) {
		self.count = 0;
		self.compressed_data = None;
	}
	fn len(&self) -> u16 {
		let x = if self.compressed_data.is_none() { 0 } else { 64 };
		return self.count + x;
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
			compressed_data: None,
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
	fn append_path_ss8(&mut self, ss: &StackStack8x64) {
		for i in 0..ss.next {
			self.push(&Move::from_u8_unchecked(ss.stack[i]));
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

impl SuperShrunkPath {
	pub fn to_path(&self) -> Vec::<Move> {
		let mut path = Vec::<Move>::with_capacity(self.len() as usize);
		for i in 0..self.count as usize {
			let block = self.data.stack[i/64];
			let shr = block >> (2*(i%64));
			path.push( Move::from_u128_unchecked( shr & 0x03 ) );
		}
		path
	}
/*	pub fn get_u(&self, i: usize) -> u128 {
		if i >= self.count as usize { panic!("ShrunkPath::get index is too high"); }
		return (self.data.stack[i/64] >> (2*(i%64))) & 0x03;
	}
	pub fn set_u(&mut self, i: usize, val: u128) {
		if i >= self.count as usize { panic!("ShrunkPath::set index is too high"); }
		let bidx = 2*(i%64);
		let mask: u128 = ! ( 0x03 << bidx );
		let maskdata = self.data.stack[i/64] & mask;
		let newdata = maskdata | (val << bidx);
		self.data.stack[i/64] = newdata;
	} *//*	pub fn pop(&mut self) -> Move {
		if self.count == 0 { panic!("stack underflow in ShrunkPath::pop"); }
		self.count -= 1;
		let i = self.count as usize;
		let block = self.data.stack[i/64];
		let shr = block >> (2*(i%64));
		return Move::from_u128_unchecked( shr & 0x03 );
	} */
/*	pub fn reverse(&mut self) {
		if self.count < 2 { return; }
		for i in 0..(self.count as usize / 2) {
			let revi = self.count as usize - i - 1;
			let iv = self.get_u(i);
			let rv = self.get_u(revi);
			self.set_u(i, rv);
			self.set_u(revi, iv);
		}		
	} */
	/* pub fn with_capacity(_c: usize) -> Self {
		Self {
			prefix_ptr: core::ptr::null_mut(),
			prefix_path: None,
			count: 0,
			data: StackStack128::new(),
		}
	} */	
	
}