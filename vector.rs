// Box Bopper: Sokoban-like game
// Copyright David Atkinson 2020-2021
//
// vector.rs: has vector for points / moves / directions and paths
//
// A point and a direction can both be implemented as a Vector

use wasm_bindgen::prelude::*;
use js_sys::Array;

use crate::stackstack::{StackStack8x64,StackStack};

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
	pub fn from_u128_unchecked(n: u128) -> Move {
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

