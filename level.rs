

use wasm_bindgen::prelude::*;
use js_sys::{Array,JsString};

use std::convert::TryInto;
use std::collections::HashMap;
use std::string::String;

use crate::vector::Vector;
use super::Obj;
use crate::builtins::BUILTIN_LEVELS;

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
	keyvals: HashMap::<String,String>,
	pub w: u16,
	pub h: u16,
	pub human_pos: Vector,
	noboxx_pts: Vec::<Vector>,
	boxx_pts: Vec::<Vector>,
	data: Vec::<Obj>,
}


#[derive(Clone,PartialEq,PartialOrd)]
pub struct SpLevel {		/* special level for solving */
	pub w: u16,
	pub human_pos: Vector,
	pub data: Vec::<Obj>,
}


impl SpLevel {
	pub fn from_level(level: &Level) -> Self {		
		Self {
			w: level.w,
			human_pos: level.human_pos.clone(),
			data: level.data.clone(),
		}
	}
	pub fn get_obj_at_pt(&self, pt: &Vector) -> Obj {
		self.data[(pt.0 as usize) + (pt.1 as usize) * (self.w as usize)]
	}
	pub fn set_obj_at_pt(&mut self, pt: &Vector, obj: Obj) {
		self.data[(pt.0 as usize) + (pt.1 as usize) * (self.w as usize)] = obj;
	}	
	pub fn get_obj_at_pt_checked(&self, pt: &Vector) -> Obj {
		let h: u16 = (self.data.len() / self.w as usize) as u16;
		if pt.0 < 0 || pt.0 >= self.w as i32 || pt.1 < 0 || pt.1 >= h as i32 {
			Obj::Wall
		} else {
			self.data[(pt.0 as usize) + (pt.1 as usize) * (self.w as usize)]
		}
	}
	pub fn set_obj_at_pt_checked(&mut self, pt: &Vector, obj: Obj) {
		let h: u16 = (self.data.len() / self.w as usize) as u16;
		if pt.0 < 0 || pt.0 >= self.w as i32 || pt.1 < 0 || pt.1 >= h as i32 {
			panic!("set_obj_at_pt_checked(): out of bounds pt");
		} else {
			self.data[(pt.0 as usize) + (pt.1 as usize) * (self.w as usize)] = obj;
		}
	}	
	pub fn have_win_condition(&self) -> bool {
		for obj in self.data.iter() {
			match obj {
				Obj::Boxx | Obj::Hole | Obj::HumanInHole => return false,
				_ => {},
			};
		}
		return true;
	}
	pub fn eq_data(&self, b: &SpLevel) -> bool {
		self.data == b.data && self.human_pos == b.human_pos
	}	
	pub fn to_string(&self) -> String {
		let mut s = String::new();
		for _ in 0..self.w+2 { s+="#"; }
		s += "\n";
		for y in 0..(self.data.len()/self.w as usize) {
			s += "#";
			for x in 0..self.w as usize {
				s += &self.get_obj_at_pt(&Vector(x as i32,y as i32)).to_char().to_string();
			}
			s += "#\n";
		}
		for _ in 0..self.w+2 { s+="#"; }
		s += "\n";
		s
	}	
}

#[wasm_bindgen]
impl Level {
	pub fn clone(&self) -> Level {
		Level {
			keyvals: self.keyvals.clone(),
			w: self.w,
			h: self.h,
			human_pos: self.human_pos.clone(),
			boxx_pts: self.boxx_pts.clone(),
			noboxx_pts: self.noboxx_pts.clone(),
			data: self.data.clone(),
		}
	}
	#[wasm_bindgen]
	pub fn from_builtin(number: usize) -> Option<Level> {
		// locate string
		if number >= BUILTIN_LEVELS.len() {
			return None;
		}
		
		let level = BUILTIN_LEVELS[number];
		let level = Level::from_str(level);
		level
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
/*	pub fn get_level_width(&self) -> u32 {
		self.level.w as u32
	}
	
	pub fn get_level_height(&self) -> u32 {
		self.level.h as u32
	} */
	pub fn get_title(&self) -> JsString {
		let s = self.keyvals.get(&"title".to_string());
		let s2 = s.unwrap_or(&"untitled".to_string()).to_string();
		return JsString::from(s2);
	}
	pub fn have_win_condition(&self) -> bool {
		for obj in self.data.iter() {
			match obj {
				Obj::Boxx | Obj::Hole | Obj::HumanInHole => return false,
				_ => {},
			};
		}
		return true;
	}

}

// non-js
impl Level {
	pub fn from_str(level_str: &str) -> Option<Level> {
		let mut count: usize = 0;
		let mut h: u16 = 0;
		let mut w: u16 = 0;
		let mut data = Vec::<Obj>::with_capacity(128);
		let mut human_pos: Vector = Vector(-1,-1);
		let mut keyvals = HashMap::new();
		let mut kvmode = false;
	
		for line in level_str.lines() {		
	/*		let txt = match line {
				Ok(o) => o,
				_ => panic!("Failed to read line from level string."),
			};*/
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
						human_pos = Vector(i.try_into().unwrap(),h.try_into().unwrap());
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
		let human_pos = human_pos.add(&Vector(-1,-1));
		println!("Dimensions: {} x {}", w, h);
		
		if human_pos.0 == -1 || human_pos.1 == -1 {
			panic!("Human not found in level");
		}
		
		println!("Human at: {}, {}", human_pos.0, human_pos.1);
		
		if w < 1 || h < 1 {
			panic!("Width and Height must be >= 2");
		}
		
		let mut level = Level {
			keyvals: keyvals,
			w: w,
			h: h,
			human_pos: human_pos,
			noboxx_pts: Vec::new(),
			boxx_pts: Vec::new(),
			data: data,
		};
		level.do_noboxx_pts();
		level.do_boxx_pts();
		return Some(level);
	}
	pub fn from_file(filename: &str) -> Option<Level> {
		let input = std::fs::read_to_string(filename);
		let input = match input {
			Ok(x) => x,
			_ => panic!("Failed to open level file: {}", filename),
		};
			
		let level = Level::from_str(&input);
		level

	}
	
	pub fn from_parts(title: String, w: u16, h: u16, human_pos: Vector, data: Vec::<Obj>) -> Level {
		let level = Level {
			keyvals: HashMap::from( [("title".to_string(),title)].iter().cloned().collect() ),
			w: w as u16,
			h: h as u16,
			human_pos: human_pos,
			noboxx_pts: Vec::new(),
			boxx_pts: Vec::new(),
			data: data,
		};
		level
	}
	pub fn get_title_str(&self) -> String {
		return self.keyvals.get(&"title".to_string()).unwrap_or(&"untitled".to_string()).to_string();
	}
	pub fn contains_key(&self, key: &String) -> bool {
		self.keyvals.contains_key(key)
	}
	pub fn get_keyval(&self, key: &str) -> String {
		let s = self.keyvals.get(key).unwrap();
		s.to_string()
	}
	pub fn set_keyval(&mut self, key: &str, val: &str) {
		self.keyvals.insert(key.to_string(),val.to_string());
	}
	pub fn clear_human(&mut self) {
		// clear the human from the level to make certain things easier
		let pt = self.human_pos;
		let obj = self.get_obj_at_pt(&self.human_pos);
		let obj2 = match obj {
			Obj::Human => Obj::Space,
			Obj::HumanInHole => Obj::Hole,
			_ => panic!("Human not where it should be"),
		};
		self.set_obj_at_pt(&pt, obj2);
	}
	pub fn place_human(&mut self) {
		// place the human in the level data
		let pt = self.human_pos;
		let obj = self.get_obj_at_pt(&self.human_pos);
		let obj2 = match obj {
			Obj::Space => Obj::Human,
			Obj::Hole => Obj::HumanInHole,
			_ => panic!("Human cannot be there!"),
		};
		self.set_obj_at_pt(&pt, obj2);
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
		};

		let mut start_x: Option<isize> = None;
		let mut halls = Vec::<HallInfoH>::new();

		// find the hall '#   '+
		for y in -1..=self.h as isize {
			for x in -1..=self.w as isize {
				let obj_here = self.get_obj_at_pt_checked(&Vector(x as i32,y as i32));
				print!("{}",obj_here.to_char());
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
			println!();
		} 
		
		println!("len of halls: {}", halls.len());
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
		};
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
		println!("len of halls: {}", halls.len());

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
		print!("noboxx_pts: ");
		for p in &self.noboxx_pts {
			print!("{} ",p.to_string());
		}
		println!(""); 
		self.place_human();
	}
	pub fn do_boxx_pts(&mut self) {
		let mut pts: Vec::<Vector> = Vec::new();
		for y in 0..self.h {
			for x in 0..self.w {
				let pt = Vector(x.try_into().unwrap(),y.try_into().unwrap());
				let obj = self.get_obj_at_pt(&pt);
				if obj == Obj::Boxx || obj == Obj::BoxxInHole {
					pts.push(pt);
				}
			}
		}
		self.boxx_pts = pts;
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
		self.noboxx_pts.contains(&v)
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
	pub fn get_noboxx_pts(&self) -> &Vec<Vector> {
		&self.noboxx_pts
	}
	pub fn get_boxx_pts(&self) -> &Vec<Vector> {
		return &self.boxx_pts;
	}
	pub fn vector_in_bounds(&self, v: &Vector) -> bool {
		v.0 >= 0 && v.0 < (self.w as i32) && v.1 >= 0 && v.1 < (self.h as i32)
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
}





