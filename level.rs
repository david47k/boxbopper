// Box Bopper: Sokoban-like game
// Copyright David Atkinson 2020-2021
//
// level.rs: store level data and perform basic operations

use wasm_bindgen::prelude::*;
use js_sys::{Array,JsString};

use std::convert::TryInto;
use std::collections::HashMap;
use std::string::String;

use crate::vector::{Vector,VectorSm};
use super::Obj;
use crate::builtins::BUILTIN_LEVELS;

pub fn verify_builtins() -> bool {
	// check that num matches index
	let mut ok = true;
	for i in 0..BUILTIN_LEVELS.len() {
		let level = Level::from_builtin(i);
		match level {
			Ok(_) => {},
			Err(s) => {
				println!("Level index {} is invalid: {}", i, s);
				ok = false;
				continue;
			},
		}
		let level = level.unwrap();
		if level.contains_key("num") {
			let n = level.get_keyval("num").parse::<u16>().unwrap();
			if n as usize != i {
				println!("Level index {} has mismatched num {}", i, n);
				ok = false;
			}
		} else {
			println!("Warning: Level index {} has no num.", i);
		}
	}
	ok
}

#[derive(Copy,Clone,PartialEq,PartialOrd,Ord,Eq,Hash)]
pub struct CmpData {
	pub human_x: i8,
	pub human_y: i8,
	pub blocks: [u64; 4],
}

impl CmpData {
	pub fn new() -> CmpData { 
		CmpData {
			human_x: 0,
			human_y: 0,
			blocks: [0_u64; 4],
		}
	}
	pub fn from_data(human_pos: &Vector, ldata: &Vec::<Obj>) -> CmpData {
		let mut cmp_data = CmpData::new();
		cmp_data.human_x = human_pos.0 as i8;
		cmp_data.human_y = human_pos.1 as i8;
		let mut data: u64 = 0;
		let mut bits_used: usize = 0;
		let mut block = 0;
		for o in ldata.iter() {
			if bits_used % 64 == 0 && bits_used != 0 {
				cmp_data.blocks[block] = data;
				block += 1;
				data = 0;
			}
			data <<= 1;
			data |= (*o==Obj::Boxx||*o==Obj::BoxxInHole) as u64;
			bits_used += 1;
		}
		// align last block
		if bits_used % 64 != 0 {
			data = data << (64 - (bits_used % 64));
		}
		cmp_data.blocks[block] = data;
		
		if false {
			println!("cmp data:");
			for block in cmp_data.blocks.iter() {
				for i in 0..63 {
					print!("{}", (block >> (63-i)) & 1);
				}
				println!();
			}
		}
		
		cmp_data
	}
}


const X1VARS: [[Vector;3];8] = // only interested in the neighbours (not the opposite)
[
[Vector(0,0),Vector(0,1),Vector(1,0)],
[Vector(0,0),Vector(1,0),Vector(0,1)],
[Vector(0,1),Vector(0,0),Vector(1,1)],
[Vector(0,1),Vector(1,1),Vector(0,0)],
[Vector(1,0),Vector(0,0),Vector(1,1)],
[Vector(1,0),Vector(1,1),Vector(0,0)],
[Vector(1,1),Vector(0,1),Vector(1,0)],
[Vector(1,1),Vector(1,0),Vector(0,1)],
];


#[wasm_bindgen]
#[derive(Clone,PartialEq)] //,PartialOrd
pub struct Level {
	pub w: u16,
	pub h: u16,
	pub human_pos: Vector,
	win_data: [u64; 4],
	data: Vec::<Obj>,
	keyvals: HashMap::<String,String>,
	noboxx_pts: Vec::<Vector>,
	boxx_pts: Vec::<Vector>,
	hole_pts: Vec::<Vector>,
	wall_pts: Vec::<Vector>,
	cleared_of_human: bool,
}


#[derive(Clone,PartialEq)]
pub struct SpLevel {
	pub w: i8,
	pub h: i8,
	pub cmp_data: CmpData,
}


impl SpLevel {
	pub fn from_level(level: &Level) -> Self {		
		Self {
			w: level.w as i8,
			h: level.h as i8,
			cmp_data: CmpData::from_data(&level.human_pos, &level.data),
		}
	}
	pub fn to_level(&self, base_level: &Level) -> Level {
		let mut level = base_level.clone();

		level.data.clear();
		for y in 0..(self.h as i32) {
			for x in 0..(self.w as i32) {
				level.data.push(self.get_obj_at_pt(&Vector(x, y), base_level));
			}
		}
		
		level.human_pos = Vector(self.cmp_data.human_x as i32, self.cmp_data.human_y as i32);
		level.cleared_of_human = false;
		level
	}	
	pub fn get_obj_at_pt(&self, pt: &Vector, base_level: &Level) -> Obj {
		// THIS IS A SLOW FUNCTION...
		let cmp_data = &self.cmp_data;
		let idx_bits: usize = pt.0 as usize + pt.1 as usize * self.w as usize;
		let is_boxx = (cmp_data.blocks[idx_bits/64] & (0x8000_0000_0000_0000 >> (idx_bits%64))) != 0;
		let is_human = pt.0 == self.cmp_data.human_x as i32 && pt.1 == self.cmp_data.human_y as i32;
		let base_obj = base_level.get_obj_at_pt(pt);

		match (base_obj, is_human, is_boxx) {
			(Obj::Hole,true,_)  => Obj::HumanInHole,
			(Obj::Hole,_,true)  => Obj::BoxxInHole,
			(Obj::Hole,_,_)     => Obj::Hole,
			(Obj::Space,true,_) => Obj::Human,
			(Obj::Space,_,true) => Obj::Boxx,
			(Obj::Space,_,_)    => Obj::Space,
			(Obj::Wall,_,_)     => Obj::Wall,
			(_,_,_)             => panic!("WEirdness in SpLevel::get_obj_at_pt"),
		}
	}
	pub fn get_obj_at_pt_nohuman(&self, pt: &Vector, base_level: &Level) -> Obj {
		let cmp_data = &self.cmp_data;
		let idx_bits: usize = pt.0 as usize + pt.1 as usize * self.w as usize;
		let is_boxx = (cmp_data.blocks[idx_bits/64] & (0x8000_0000_0000_0000 >> (idx_bits%64))) != 0;
		let base_obj = base_level.get_obj_at_pt(pt);

		match (base_obj, is_boxx) {
			(Obj::Hole,true)  => Obj::BoxxInHole,
			(Obj::Hole,_)     => Obj::Hole,
			(Obj::Space,true) => Obj::Boxx,
			(Obj::Space,_)    => Obj::Space,
			(Obj::Wall,_)     => Obj::Wall,
			(_,_)             => panic!("WEirdness in SpLevel::get_obj_at_pt_nohuman"),
		}
	}	
	pub fn is_boxx_at_pt(&self, pt: &Vector) -> bool {
		// ignores underlying level
		let cmp_data = &self.cmp_data;
		let idx_bits: usize = pt.0 as usize + pt.1 as usize * self.w as usize;
		let is_boxx = (cmp_data.blocks[idx_bits/64] & (0x8000_0000_0000_0000 >> (idx_bits%64))) != 0;
		is_boxx
	}
	pub fn set_boxx_at_pt(&mut self, pt: &Vector) {
		// ignores underlying level
		let cmp_data = &mut self.cmp_data;
		let idx_bits: usize = pt.0 as usize + pt.1 as usize * self.w as usize;
		cmp_data.blocks[idx_bits/64] |= 0x8000_0000_0000_0000 >> (idx_bits%64);
	}
	pub fn clear_boxx_at_pt(&mut self, pt: &Vector) {
		// ignores underlying level
		let cmp_data = &mut self.cmp_data;
		let idx_bits: usize = pt.0 as usize + pt.1 as usize * self.w as usize;
		let and_val = !(0x8000_0000_0000_0000 >> (idx_bits%64));		
		cmp_data.blocks[idx_bits/64] &= and_val;
	}
	pub fn set_human_pos(&mut self, pt: &Vector) {
		// ignores underlying level
		let cmp_data = &mut self.cmp_data;
		cmp_data.human_x = pt.0 as i8;
		cmp_data.human_y = pt.1 as i8;
	}
	pub fn get_human_pos(&self) -> Vector {
		// ignores underlying level
		Vector(self.cmp_data.human_x as i32, self.cmp_data.human_y as i32)
	}
	pub fn get_obj_at_pt_checked(&self, pt: &Vector, base_level: &Level) -> Obj {
		//if pt.0 < 0 || pt.0 >= self.w as i32 || pt.1 < 0 || pt.1 >= self.h as i32 {			// slower version
		if ( pt.0 | pt.1 | (self.w as i32 - pt.0 - 1) | (self.h as i32 - pt.1 - 1)  ) < 0 {		// faster version
			Obj::Wall
		} else {
			self.get_obj_at_pt(pt, base_level)
		} 
	} 
	pub fn get_obj_at_pt_nohuman_checked(&self, pt: &Vector, base_level: &Level) -> Obj {
		//if pt.0 < 0 || pt.0 >= self.w as i32 || pt.1 < 0 || pt.1 >= self.h as i32 {			// slower version
		if ( pt.0 | pt.1 | (self.w as i32 - pt.0 - 1) | (self.h as i32 - pt.1 - 1)  ) < 0 {		// faster version
				Obj::Wall
		} else {
			self.get_obj_at_pt_nohuman(pt, base_level)
		} 
	} 
	pub fn have_win_condition(&self, base_level: &Level) -> bool {
		self.cmp_data.blocks == base_level.win_data
	}
}



#[wasm_bindgen]
impl Level {
	#[wasm_bindgen]
	pub fn from_builtin_js(number: usize) -> Result<Level, JsValue> {
		let level = Level::from_builtin(number);
		return match level {
			Ok(l) => Ok(l),
			Err(s) => Err(JsValue::from_str(s)),
		};
	}
	pub fn get_obj_at_pt(&self, pt: &Vector) -> Obj {
		self.data[(pt.0 as usize) + (pt.1 as usize) * (self.w as usize)]
	}
	pub fn get_obj_at_idx(&self, idx: usize) -> Obj {
		self.data[idx]
	}
	pub fn set_obj_at_pt(&mut self, pt: &Vector, obj: Obj) {
		self.data[(pt.0 as usize) + (pt.1 as usize) * (self.w as usize)] = obj;
	}	
	pub fn set_obj_at_idx(&mut self, idx: usize, obj: Obj) {
		self.data[idx] = obj;
	}
	pub fn get_obj_at_pt_checked(&self, pt: &Vector) -> Obj {
		if pt.0 < 0 || pt.0 >= self.w as i32 || pt.1 < 0 || pt.1 >= self.h as i32 {
			Obj::Wall
		} else {
			self.data[(pt.0 as usize) + (pt.1 as usize) * (self.w as usize)]
		}
	}
	pub fn set_obj_at_pt_checked(&mut self, pt: &Vector, obj: Obj) {
		if pt.0 < 0 || pt.0 >= self.w as i32 || pt.1 < 0 || pt.1 >= self.h as i32 {
			panic!("set_obj_at_pt_checked(): out of bounds pt");
		} else {
			self.data[(pt.0 as usize) + (pt.1 as usize) * (self.w as usize)] = obj;
		}
	}	
	pub fn get_data(&self) -> Array {
		self.data.clone().into_iter().map(|obj| JsValue::from(obj as u32)).collect()
	}
	fn get_vslice(&self, x: isize, y0: isize, y1: isize) -> Vec::<Obj> {
		let mut rv = Vec::<Obj>::new();
		for i in y0..y1 {
			let v = Vector(x as i32,i as i32);
			if self.vector_in_bounds(&v) {
				rv.push(self.get_obj_at_pt(&Vector(x as i32,i as i32)));
			} else {
				rv.push(Obj::Wall);
			}
		}
		return rv;
	}
	fn get_hslice(&self, x0: isize, x1: isize, y: isize) -> Vec::<Obj> {
		let mut rv = Vec::<Obj>::new();
		for i in x0..x1 {
			let v = Vector(i as i32,y as i32);
			if self.vector_in_bounds(&v) {
				rv.push(self.get_obj_at_pt(&Vector(i as i32,y as i32)));
			} else {
				rv.push(Obj::Wall);
			}
		}
		return rv;
	}
	pub fn get_level_width(&self) -> u32 {
		self.w as u32
	}
	pub fn get_level_height(&self) -> u32 {
		self.h as u32
	} 
	pub fn get_title(&self) -> JsString {
		let s = self.keyvals.get(&"title".to_string());
		let s2 = s.unwrap_or(&"untitled".to_string()).to_string();
		return JsString::from(s2);
	}
	pub fn have_win_condition(&self) -> bool {
		for obj in self.data.iter() {
			match obj {
				Obj::Boxx | Obj::Hole => return false,
				_ => {},
			};
		}
		return true;
	}
}

// non-js
impl Level {
	pub fn from_builtin(number: usize) -> Result<Level, &'static str> {
		// locate string
		if number >= BUILTIN_LEVELS.len() {
			return Err("Level number too high");
		}
		
		let level = BUILTIN_LEVELS[number];
		Level::from_str(level)
	}
	pub fn from_str(level_str: &str) -> Result<Level, &str> {
		let mut count: usize = 0;
		let mut h: u16 = 0;
		let mut w: u16 = 0;
		let mut data = Vec::<Obj>::with_capacity(128);
		let mut human_pos: Option<Vector> = None;
		let mut keyvals = HashMap::new();
		let mut kvmode = false;
		let mut num_boxxes = 0;
		let mut num_holes = 0;
	
		for line in level_str.lines() {		
			let txt = line;
			if count == 0 {
				// read in length
				w = txt.len() as u16;			
			}
			// check length equal to w
			if !kvmode && txt.len() == w as usize {	
				// split line into characters
				for (i,c) in txt.char_indices() {		// chars() is iterator
					if c == '&' || c == '%' {
						// found human_pos
						if human_pos.is_none() {
							human_pos = Some(Vector(i.try_into().unwrap(),h.try_into().unwrap()));
						} else {
							return Err("More than one human found!");
						}
					}
					if c == 'O' || c == '%' || c == '@' {
						num_holes += 1;
					}
					if c == '*' || c == '@' {
						num_boxxes += 1;
					}
					data.push( Obj::from_char(&c) );
				}
				h += 1;
			} else {
				kvmode = true;
				// read in key and vals
				// left of ':', right of ':', strip whitespace at front and end
				if txt.len() >= 2 {
					let idx = txt.find(':');
					if idx.is_some() {
						let idx = idx.unwrap();
						let left = &txt[0..idx].trim();
						let right = &txt[idx+1..].trim();
						if left.len() > 0 {
							keyvals.insert(left.to_string(),right.to_string());
						}
					}
				}
			}
			
			count += 1;
		}
	
		// remove the borders
		let mut tdata = Vec::<Obj>::new();
		for y in 1..(h-1) as usize {
			for x in 1..(w-1) as usize {
				tdata.push(data[y*w as usize+x]);
			}
		}
		let data = tdata;		
		w -= 2;
		h -= 2;
		if human_pos.is_none() {
			return Err("Human not found in level!");
		}
		let	human_pos = human_pos.unwrap().add(&Vector(-1,-1));

		if w < 1 || h < 1 {
			//println!("Dimensions: {} x {}", w, h);
			return Err("Width and Height must be at least 1!");
		}
		if w > 127 || h > 127 || w * h > 256 {
			//println!("Dimensions: {} x {}", w, h);
			return Err("Level too big! Maximum width 127. Maximum height 127. Maximum width * height 256.");
		} 		

		// Check for unequal boxes / holes
		if num_boxxes != num_holes {
			return Err("Num boxes is not equal to num holes!");
		}

		if num_boxxes < 1 {
			return Err("Must be at least one box!");
		}

		if num_boxxes > 24 {		// This is an arbitrary limit, but currently too many boxes uses too many resources
			return Err("Too many boxes! (Maximum 24)");
		}

		let mut level = Level {
			keyvals: keyvals,
			w: w,
			h: h,
			human_pos: human_pos,
			noboxx_pts: Vec::new(),
			boxx_pts: Vec::new(),
			hole_pts: Vec::new(),
			wall_pts: Vec::new(),
			win_data: [0_u64; 4],
			data: data,
			cleared_of_human: false,
		};
		level.make_win_data();
		level.do_noboxx_pts();
		level.do_boxx_pts();
		return Ok(level);
	}
	pub fn from_file(filename: &str) -> Result<Level, String> {
		let input = std::fs::read_to_string(filename);
		let input = match input {
			Ok(x) => x,
			_ => return Err("Failed to open level file.".to_string()),
		};
			
		let level = Level::from_str(&input);
		return match level {
			Ok(l) => Ok(l),
			Err(s) => Err(s.to_string()),
		};
	}
	pub fn from_parts(title: String, w: u16, h: u16, human_pos: Vector, data: Vec::<Obj>) -> Level {
		let mut level = Level {
			keyvals: HashMap::from( [("title".to_string(),title)].iter().cloned().collect() ),
			w: w as u16,
			h: h as u16,
			human_pos: human_pos,
			noboxx_pts: Vec::new(),
			boxx_pts: Vec::new(),
			hole_pts: Vec::new(),
			wall_pts: Vec::new(),
			data: data,
			win_data: [0_u64; 4],
			cleared_of_human: false,
		};
		if level.confirm_no_human() {
			level.place_human();
		}
		level.do_noboxx_pts();
		level.do_boxx_pts();
		level.make_win_data();

		level
	}
	pub fn get_title_str(&self) -> String {
		return self.keyvals.get(&"title".to_string()).unwrap_or(&"untitled".to_string()).to_string();
	}
	pub fn contains_key(&self, key: &str) -> bool {
		self.keyvals.contains_key(key)
	}
	pub fn get_keyval(&self, key: &str) -> String {
		let s = self.keyvals.get(key).unwrap();
		s.to_string()
	}
	pub fn get_keyval_or(&self, key: &str, ors: &str) -> String {
		let s = self.keyvals.get(key);
		if s.is_some() {
			return s.unwrap().to_string();
		}
		ors.to_string()
	}
	pub fn set_keyval(&mut self, key: &str, val: &str) {
		self.keyvals.insert(key.to_string(),val.to_string());
	}
	pub fn confirm_no_human(&self) -> bool {
		for y in 0..self.h as i32 {
			for x in 0..self.w as i32 {
				match self.get_obj_at_pt(&Vector(x,y)) {
					Obj::Human|Obj::HumanInHole => { 
						return false; 
					},
					_ => {},
				}
			}
		}
		true
	}
	pub fn clear_human(&mut self) {
		if self.cleared_of_human {
			panic!("clear_human() called twice!");
		}

		// clear the human from the level to make certain things easier
		let pt = self.human_pos;
		let obj = self.get_obj_at_pt(&pt);
		let obj2 = match obj {
			Obj::Human => Obj::Space,
			Obj::HumanInHole => Obj::Hole,
			_ => panic!("Human not where it should be ({},{})",pt.0,pt.1),
		};
		self.set_obj_at_pt(&pt, obj2);

		if !self.confirm_no_human() {
			panic!("confirm_no_human() failed")
		}

		self.cleared_of_human = true;
	}
	pub fn clear_human_cloned(&self) -> Level {
		let mut level = self.clone();
		level.clear_human();
		level
	}
	pub fn clear_boxxes_cloned(&self) -> Level {
		let mut level = self.clone();
		level.clear_boxxes();
		level
	}
	pub fn place_human(&mut self) {
		if !self.cleared_of_human {
			panic!("place_human() called but level has not been cleared");
		}
		if !self.confirm_no_human() {
			panic!("Human found before place_human()!");
		}
		
		// place the human in the level data
		let pt = self.human_pos;
		let obj = self.get_obj_at_pt(&self.human_pos);
		let obj2 = match obj {
			Obj::Space => Obj::Human,
			Obj::Hole => Obj::HumanInHole,
			_ => panic!("Human cannot be there!"),
		};
		self.set_obj_at_pt(&pt, obj2);
		if self.confirm_no_human() {
			panic!("Human NOT found AFTER place_human({},{})!",pt.0,pt.1);
		}
		self.cleared_of_human = false;
	}
	pub fn clear_boxxes(&mut self) {
		// clear the boxxes from the level to make certain things easier
		for o in self.data.iter_mut() {
			if *o == Obj::Boxx { *o = Obj::Space; }
			else if *o == Obj::BoxxInHole { *o = Obj::Hole; }
		}
	}
	pub fn do_noboxx_pts(&mut self) {
		let mut noboxx_pts: Vec::<Vector> = Vec::new();
		// aside from #, there are some points where box's simply can't go
		// e.g. the 2x2 [*#][# ] in any orientation (where space could be human too)

		self.clear_human();

		let block_match = [Obj::Space,Obj::Wall,Obj::Wall];
		for y in -1..=self.h as isize {
			for x in -1..=self.w as isize {
				for z in &X1VARS {
					let pt = Vector(x as i32,y as i32);
					let objs = [ self.get_obj_at_pt_checked(&pt.add(&z[0])), 
								 self.get_obj_at_pt_checked(&pt.add(&z[1])), 
								 self.get_obj_at_pt_checked(&pt.add(&z[2])), ];
					if objs == block_match {
						noboxx_pts.push(pt.add(&z[0]));
					} 
				}
			}
		} 

		self.noboxx_pts = noboxx_pts;
	
		// walls that follow the following pattern also can't have boxxes
		//  ##...# 
		// #      #

		let hall_start = vec![ Obj::Wall, Obj::Space, Obj::Space, Obj::Space ];
		let hall_len: isize = 4;

		struct HallInfoH {
			x: isize,
			y: isize,
			end_x: isize,
		}

		let mut start_x: Option<isize> = None;
		let mut halls = Vec::<HallInfoH>::new();

		// find the hall '#   '+
		for y in -1..=self.h as isize {
			for x in -1..=self.w as isize {
				let obj_here = self.get_obj_at_pt_checked(&Vector(x as i32,y as i32));
				if start_x.is_some() && obj_here == Obj::Space { 					// Continuation of hallway
					// do nothing
				} else if start_x.is_some() && obj_here == Obj::Wall {				// We have end of the hall					
					halls.push( HallInfoH { x:start_x.unwrap(), y: y, end_x:x } );
					start_x = None;
				} else if start_x.is_some() {										// Not a real hallway
					start_x = None;
				}
				if start_x.is_none() {
					if hall_start.starts_with(&self.get_hslice(x, x + hall_len, y)) {
						start_x = Some(x);
					}
				}
			}
		} 
		
		// check if the hall is a valid hall (has a complete wall on one side)
		for h in halls {
			let range1 = self.get_hslice(h.x+1,h.end_x,h.y-1);
			let range2 = self.get_hslice(h.x+1,h.end_x,h.y+1);
			if range1.iter().all(|o| o==&Obj::Wall) || range2.iter().all(|o| o==&Obj::Wall) {
				(h.x+1..h.end_x).into_iter().for_each( |x| self.noboxx_pts.push(Vector(x as i32, h.y as i32)));
			}
		}

		// now do it all vertically!
		struct HallInfoV {
			x: isize,
			y: isize,
			end_y: isize,
		}
		let mut start_y: Option<isize> = None;
		let mut halls = Vec::<HallInfoV>::new();

		// find the hall '    '+
		for x in -1..=self.w as isize {
			for y in -1..=self.h as isize {
				let obj_here = self.get_obj_at_pt_checked(&Vector(x as i32,y as i32));
				if start_y.is_some() && (obj_here == Obj::Space) { 	// Continuation of hallway
					// do nothing
				} else if start_y.is_some() && obj_here == Obj::Wall {				// We have end of the hall					
					halls.push( HallInfoV { x:x, y:start_y.unwrap(), end_y:y } );
					start_y = None;
				} else if start_y.is_some() {										// Not a real hallway
					start_y = None;
				}
				if start_y.is_none() {
					if hall_start.starts_with(&self.get_vslice(x, y, y + hall_len)) {		// Start of the hallway
						start_y = Some(y);
					}
				}
			}
		} 

		// check if the hall is a valid hall (has a complete wall on one side)
		for h in halls {
			let range1 = self.get_vslice(h.x-1, h.y+1, h.end_y);
			let range2 = self.get_vslice(h.x+1, h.y+1, h.end_y);
			if range1.iter().all(|o| o==&Obj::Wall) || range2.iter().all(|o| o==&Obj::Wall) {
				(h.y+1..h.end_y).into_iter().for_each( |y| self.noboxx_pts.push(Vector(h.x as i32, y as i32)));
			}
		}

		self.noboxx_pts.sort_unstable();
		self.noboxx_pts.dedup();
		if false {
			print!("noboxx pts: ");
			for p in &self.noboxx_pts {
				print!("{} ",p.to_string());
			}
			println!("");
		}
		self.place_human();
	}
	pub fn do_boxx_pts(&mut self) {
		let mut bpts: Vec::<Vector> = Vec::new();
		let mut hpts: Vec::<Vector> = Vec::new();
		let mut wpts: Vec::<Vector> = Vec::new();
		for y in 0..self.h {
			for x in 0..self.w {
				let pt = Vector(x.try_into().unwrap(),y.try_into().unwrap());
				let obj = self.get_obj_at_pt(&pt);
				if obj == Obj::Boxx || obj == Obj::BoxxInHole {
					bpts.push(pt);
				}
				if obj == Obj::BoxxInHole || obj == Obj::Hole || obj == Obj::HumanInHole {
					hpts.push(pt);
				}
				if obj == Obj::Wall {
					wpts.push(pt);
				}
			}
		}
		bpts.sort();
		hpts.sort();
		wpts.sort();
		self.boxx_pts = bpts;
		self.hole_pts = hpts;
		self.wall_pts = wpts;
	}
	pub fn get_box_count(&self) -> u32 {
		let mut count: u32 = 0;
		for y in 0..self.h {
			for x in 0..self.w {
				let pt = Vector(x.try_into().unwrap(),y.try_into().unwrap());
				let obj = self.get_obj_at_pt(&pt);
				if obj == Obj::Boxx || obj == Obj::BoxxInHole {
					count += 1;
				}
			}
		}
		count
	}
	pub fn in_noboxx_pts(&self, v: &Vector) -> bool {
		self.noboxx_pts.contains(v)
	}
	pub fn in_noboxx_pts8(&self, v: &VectorSm) -> bool {
		//self.noboxx_pts.contains(&v.intov())
		match self.noboxx_pts.binary_search(&v.intov()) {
			Ok(_) => true,
			_ => false,
		}
	}
	pub fn in_boxx_pts(&self, v: &Vector) -> bool {
		self.boxx_pts.contains(v)
	}
	pub fn in_boxx_pts8(&self, v: &VectorSm) -> bool {
		self.boxx_pts.contains(&v.intov())
	}
	pub fn in_hole_pts(&self, v: &Vector) -> bool {
		self.hole_pts.contains(v)
	}
	pub fn in_hole_pts8(&self, v: &VectorSm) -> bool {
		match self.hole_pts.binary_search(&v.intov()) {
			Ok(_) => true,
			_ => false,
		}
	}
	pub fn in_wall_pts(&self, v: &Vector) -> bool {
		self.wall_pts.contains(v)
	}
	pub fn in_wall_pts8(&self, v: &VectorSm) -> bool {
		match self.wall_pts.binary_search(&v.intov()) {
			Ok(_) => true,
			_ => false,
		}
	}
	pub fn strip_sprites(&mut self) {
		for idx in 0..(self.w * self.h) as usize {
			let obj = self.get_obj_at_idx(idx);
			let nobj = match obj {
				Obj::Human => Obj::Space,
				Obj::HumanInHole => Obj::Hole,
				Obj::Boxx => Obj::Space,
				Obj::BoxxInHole => Obj::Hole,
				_ => obj,
			};
			self.set_obj_at_idx(idx,nobj);
		}		
	}
	pub fn eq_data(&self, b: &Level) -> bool {
		self.data == b.data && self.human_pos == b.human_pos
	}
	pub fn get_boxx_pts(&self) -> &Vec<Vector> {
		&self.boxx_pts
	}
	pub fn get_noboxx_pts(&self) -> &Vec<Vector> {
		&self.noboxx_pts
	}
	pub fn get_hole_pts(&self) -> &Vec<Vector> {
		&self.hole_pts
	}
	pub fn vector_in_bounds_orig(&self, v: &Vector) -> bool {
		v.0 >= 0 && v.0 < (self.w as i32) && v.1 >= 0 && v.1 < (self.h as i32)
	}
	pub fn vector_in_bounds(&self, v: &Vector) -> bool {
		( v.0 | v.1 | (self.w as i32 - v.0 - 1) | (self.h as i32 - v.1 - 1)  ) >= 0
	}
	pub fn vector_in_bounds8(&self, v: &VectorSm) -> bool {
		v.0 >= 0 && v.0 < (self.w as i8) && v.1 >= 0 && v.1 < (self.h as i8)
	}
	pub fn force_vector_in_bounds(&self, v: &Vector) -> Vector {
		let mut v = v.clone();
		if v.0 < 0 					{ v.0 = 0; }
		if v.0 >= (self.w as i32) 	{ v.0 = (self.w as i32) - 1; }
		if v.1 < 0 					{ v.1 = 0; }
		if v.1 >= (self.h as i32) 	{ v.1 = (self.h as i32) - 1; }
		v
	}
	pub fn to_string(&self) -> String {
		let mut s = String::new();
		for _ in 0..self.w+2 { s+="#"; }
		s += "\n";
		for y in 0..self.h as usize {
			s += "#";
			for x in 0..self.w as usize {
				s += &self.get_obj_at_idx(y * self.w as usize + x).to_char().to_string();
			}
			s += "#\n";
		}
		for _ in 0..self.w+2 { s+="#"; }
		s += "\n";
		s
	}
	pub fn make_win_data(&mut self) {
		// we need to cache this part, map out where the holes are
		let mut blocks: [u64; 4] = [0_u64; 4];
		let mut data: u64 = 0; 
		let mut bits_used: usize = 0;
		let mut block = 0;
		for o in self.data.iter() {
			if bits_used % 64 == 0 && bits_used != 0 {
				blocks[block] = data;
				block += 1;
				data = 0;
			}
			data = data << 1;
			data |= (*o==Obj::Hole || *o==Obj::BoxxInHole || *o==Obj::HumanInHole) as u64;
			bits_used += 1;
		}
		// align last block
		if bits_used % 64 != 0 {
			data = data << (64 - (bits_used % 64));
		}
		blocks[block] = data;
		self.win_data = blocks;
		if false {
			println!("win data:");
			for block in self.win_data.iter() {
				for i in 0..63 {
					print!("{}", (block >> (63-i)) & 1);
				}
				println!();
			}
		}
	}
}

//unsafe impl Send for Level {};
