// Includes Vector and Move

use wasm_bindgen::prelude::*;
use js_sys::Array;

// A point and a direction can both be implemented as a Vector

#[wasm_bindgen]
#[derive(Clone, Copy, PartialEq, Ord, PartialOrd, Eq)]
pub struct Vector (pub i32, pub i32);

#[wasm_bindgen]
impl Vector {
	#[wasm_bindgen(constructor)]
	pub fn new(x: i32, y: i32) -> Vector {
		Vector(x,y)
	}

	pub fn add(&self, dir: &Vector) -> Vector {
		Vector(self.0 + dir.0, self.1 + dir.1)
	}
	pub fn double(&self) -> Vector {
		Vector(self.0 * 2, self.1 * 2)
	}
	pub fn scale_by(&self, n: i32) -> Vector {
		Vector(self.0 * n, self.1 * n)
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
    pub fn to_index(&self, width: &usize) -> usize {
		width * (self.1 as usize) + (self.0 as usize)
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
