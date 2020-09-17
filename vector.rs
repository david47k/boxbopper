// Includes Vector and Move

use wasm_bindgen::prelude::*;
use js_sys::Array;

use crate::stackstack::{StackStack8};

// A point and a direction can both be implemented as a Vector

#[wasm_bindgen]
#[derive(Clone, Copy, PartialEq, Ord, PartialOrd, Eq)]
pub struct Vector (pub i32, pub i32);

#[derive(Clone, Copy, PartialEq, Ord, PartialOrd, Eq)]
pub struct VectorSm ( pub i8, pub i8 );
impl VectorSm {
	pub fn fromv(v: &Vector) -> Self {
		Self ( v.0 as i8, v.1 as i8 )
	}
	pub fn intov(&self) -> Vector {
		Vector(self.0 as i32, self.1 as i32)
	}
	pub fn new(x: i8, y: i8) -> Self {
		Self (x,y)
	}
	pub fn add(&self, dir: &Self) -> Self {
		Self (self.0 + dir.0, self.1 + dir.1)
	}
	pub fn double(&self) -> Self {
		Self (self.0 * 2, self.1 * 2)
	}
	pub fn mul(&self, n: i8) -> Self {
		Self(self.0 * n, self.1 * n)
	}
	pub fn rotr(&self) -> Self {
		Self(self.1, -self.0)
	}
	pub fn rotl(&self) -> Self {
		Self(-self.1, self.0)
	}	
}

impl VectorSm {
	pub fn add_dir(&self, dir: &Move) -> Self {
		match dir {
			Move::Up    => Self( self.0,   self.1-1 ),
			Move::Right => Self( self.0+1, self.1   ),
			Move::Down  => Self( self.0,   self.1+1 ),
			Move::Left  => Self( self.0-1, self.1   ),
		}		
	}
	pub fn add_dir2(&self, dir: &Move) -> Self {
		match dir {
			Move::Up    => Self( self.0,   self.1-2 ),
			Move::Right => Self( self.0+2, self.1   ),
			Move::Down  => Self( self.0,   self.1+2 ),
			Move::Left  => Self( self.0-2, self.1   ),
		}		
	}
	pub fn to_index(&self, width: u16) -> usize {
		width as usize * (self.1 as usize) + (self.0 as usize)
    }
    pub fn to_usize(&self) -> (usize,usize) {
		(self.0 as usize, self.1 as usize)
	}
	pub fn to_string(&self) -> String {
		format!("({},{})",self.0,self.1)
	}
}


#[wasm_bindgen]
impl Vector {
	#[wasm_bindgen(constructor)]
	pub fn new(x: i32, y: i32) -> Vector {
		Self(x,y)
	}
	pub fn add(&self, dir: &Vector) -> Self {
		Self(self.0 + dir.0, self.1 + dir.1)
	}
	pub fn double(&self) -> Self {
		Self(self.0 * 2, self.1 * 2)
	}
	pub fn mul(&self, n: i32) -> Self {
		Self(self.0 * n, self.1 * n)
	}
	pub fn rotr(&self) -> Self {
		Self(self.1, -self.0)
	}
	pub fn rotl(&self) -> Self {
		Self(-self.1, self.0)
	}
	pub fn scale_by(&self, n: i32) -> Self {
		Self(self.0 * n, self.1 * n)
	}
	pub fn as_array(&self) -> Array {
		[ self.0, self.1 ].iter().map(|m| JsValue::from(*m)).collect()
	}
	pub fn eq(&self, a: &Vector) -> bool {
		self.0 == a.0 && self.1 == a.1
	}
}

// non-js
impl Vector {
	pub fn add_dir(&self, dir: &Move) -> Self {
		let d = *dir as i32; //1, 2, 4, 8
		Self(self.0+((d==1) as i32)-((d==3) as i32),self.1-((d==0) as i32)+((d==2) as i32))
/*		match dir {
			Move::Up    => Self( self.0,   self.1-1 ),
			Move::Right => Self( self.0+1, self.1   ),
			Move::Down  => Self( self.0,   self.1+1 ),
			Move::Left  => Self( self.0-1, self.1   ),
		}		*/
	}
	pub fn add_dir2(&self, dir: &Move) -> Self {
		match dir {
			Move::Up    => Self( self.0,   self.1-2 ),
			Move::Right => Self( self.0+2, self.1   ),
			Move::Down  => Self( self.0,   self.1+2 ),
			Move::Left  => Self( self.0-2, self.1   ),
		}		
	}
	pub fn to_index(&self, width: u16) -> usize {
		width as usize * (self.1 as usize) + (self.0 as usize)
    }
    pub fn to_usize(&self) -> (usize,usize) {
		(self.0 as usize, self.1 as usize)
	}
	pub fn to_string(&self) -> String {
		format!("({},{})",self.0,self.1)
	}
}


#[wasm_bindgen]
#[derive(Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum Move { Up=0, Right=1, Down=2, Left=3 }

impl Move {
	pub fn to_vector(&self) -> Vector {
		match self {
			Move::Up    => Vector( 0, -1 ),
			Move::Right => Vector( 1,  0 ),
			Move::Down  => Vector( 0,  1 ),
			Move::Left  => Vector(-1,  0 ),
		}
	}
	pub fn to_vector_sm(&self) -> VectorSm {
		match self {
			Move::Up    => VectorSm( 0, -1 ),
			Move::Right => VectorSm( 1,  0 ),
			Move::Down  => VectorSm( 0,  1 ),
			Move::Left  => VectorSm(-1,  0 ),
		}
	}
	pub fn to_string(&self) -> String {
		match self {
			Move::Up    => String::from("U"),
			Move::Right => String::from("R"),
			Move::Down  => String::from("D"),
			Move::Left  => String::from("L"),
		}
	}
	pub fn from_u32(n: u32) -> Option<Move> {
		match n {
			0 => Some(Move::Up),
			1 => Some(Move::Right),
			2 => Some(Move::Down),
			3 => Some(Move::Left),
			_ => None,
		}
	}	
	pub fn from_u32_unchecked(n: u32) -> Move {
		if n==0 { Move::Up }
		else if n==1 { Move::Right }
		else if n==2 { Move::Down }
		else { Move::Left }
	}
	pub fn from_u64_unchecked(n: u64) -> Move {
		if n==0 { Move::Up }
		else if n==1 { Move::Right }
		else if n==2 { Move::Down }
		else { Move::Left }
	}
	pub fn from_u8_unchecked(n: u8) -> Move {
		if n==0 { Move::Up }
		else if n==1 { Move::Right }
		else if n==2 { Move::Down }
		else { Move::Left }
	}
	pub fn from_u8(n: u8) -> Option<Move> {
		match n {
			0 => Some(Move::Up),
			1 => Some(Move::Right),
			2 => Some(Move::Down),
			3 => Some(Move::Left),
			_ => None,
		}
	}
	pub fn reverse(&self) -> Move {
		match self {
			Move::Up	=> Move::Down,
			Move::Left	=> Move::Right,
			Move::Right	=> Move::Left,
			Move::Down	=> Move::Up,
		}
	}
}

pub const ALLMOVES: [Move; 4] = [ Move::Up, Move::Right, Move::Down, Move::Left ];


#[derive(Clone)]
pub struct ShrunkPath {
	count: u16,
	data: Vec::<u64>,
}

impl ShrunkPath {
	pub fn new() -> Self {
		Self {
			count: 0,
			data: Vec::<u64>::new(),
		}
	}
	pub fn with_capacity(c: usize) -> Self {
		Self {
			count: 0,
			data: Vec::<u64>::with_capacity(c/32+1),
		}
	}
	pub fn len(&self) -> u16 {
		self.count
	}
	pub fn from_path(path: &Vec::<Move>) -> Self {
		let mut data = Vec::<u64>::new();
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
	pub fn push(&mut self, move1: &Move) {
		if self.count%32==0 { 
			// append new block
			self.data.push(*move1 as u64);
		} else {
			// modify existing block
			let idx = self.count as usize/32;
			let mut x = self.data[idx];
			x |= (*move1 as u64) << (2*(self.count%32));
			self.data[idx] = x;
		}
		self.count += 1;		
	}
	pub fn push_u8(&mut self, move1: u8) {
		let move1 = move1 as u64;
		if self.count%32==0 { 
			// append new block
			self.data.push(move1);
		} else {
			// modify existing block
			let idx = self.count as usize/32;
			let mut x = self.data[idx];
			x |= (move1) << (2*(self.count%32));
			self.data[idx] = x;
		}
		self.count += 1;		
	}
	pub fn get_u(&self, i: usize) -> u64 {
		if i >= self.count as usize { panic!("ShrunkPath::get index is too high"); }
		return (self.data[i/32] >> (2*(i%32))) & 0x03;
	}
	pub fn set_u(&mut self, i: usize, val: u64) {
		if i >= self.count as usize { panic!("ShrunkPath::set index is too high"); }
		let bidx = 2*(i%32);
		let mask: u64 = ! ( 0x03 << bidx );
		let maskdata = self.data[i/32] & mask;
		let newdata = maskdata | (val << bidx);
		self.data[i/32] = newdata;
	}
	pub fn append_path(&mut self, path: &Vec::<Move>) {
		for move1 in path {
			if self.count%32==0 { 
				// append new block
				self.data.push(*move1 as u64);
			} else {
				// modify existing block
				let idx = self.count as usize/32;
				let mut x = self.data[idx];
				x |= (*move1 as u64) << (2*(self.count%32));
				self.data[idx] = x;
			}
			self.count += 1;
		}
	}	
	pub fn append_path_ss8(&mut self, ss: &StackStack8) {
		for i in 0..ss.next {
			self.push(&Move::from_u8_unchecked(ss.stack[i]));
		}
	}
	pub fn _append_path_sp(&mut self, npath: &ShrunkPath) {	// BUGGY !!!!! OLD CODE 32-bit
		// for each block to append
		// split block at alignment point into two blocks
		// 0xBBAAAAAA			length of B is self.length%16, length of A is 15-(self.length%16)
		// 0xAAAAAA00           A gets shifted left appropriate amount
		// 0x000000BB           B gets shifted right appropriate amount
		// 0xAAAAdddd			A gets placed on existing data
		// 0x0000BBBB			A new block gets added for B
		// repeat
		// set our count		self.count += path.count
		if npath.count == 0 { return; }
		for ni in 0..=(npath.count/16) as usize {
			if ni == (npath.count/16) as usize && npath.count%16 == 0 { 
				return 
			};
			let b_len = self.count%16;		// 0 to 15 i.e. might be all zeros if empty
			let a_len = 16-b_len;			// 16 to 1 i.e. always exists, may take up entire u32
			let b_shr = 32-(2*b_len); 		// will shr between 2 (len=15) and 32 (len=0)
			let a_shl = 32-(2*a_len); 		// will shl between 30 (len=2) and 0  (len=16)
			let block_a = npath.data[ni] << a_shl;
			let block_b = npath.data[ni] >> b_shr;
			if self.count % 16 == 0 { // perfect alignment, append new block, i.e. b_len will have length zero!
				self.data.push(block_a);
			} else {
				self.data[self.count as usize/16] |= block_a;
				self.data.push(block_b);
			}
			self.count += if ni == (npath.count as usize/16) { npath.count%16 } else { 16 };
		}
		
	}
	pub fn to_path(&self) -> Vec::<Move> {
		let mut path = Vec::<Move>::with_capacity(self.count as usize);
		for i in 0..self.count as usize {
			let block = self.data[i/32];
			let shr = block >> (2*(i%32));
			path.push( Move::from_u64_unchecked( shr & 0x03 ) );
		}

		path
	}
	pub fn pop(&mut self) -> Move {
		if self.count == 0 { panic!("stack underflow in ShrunkPath::pop"); }
		self.count -= 1;
		let i = self.count as usize;
		let block = self.data[i/32];
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
	pub fn to_string(&self) -> String {
		let path = self.to_path();
		let mut s: String = "".to_string();
		for m in path.iter() {
			s = s + &m.to_string();
		}
		return s;
	}
}