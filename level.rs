

use wasm_bindgen::prelude::*;
use js_sys::{Array,JsString};

use std::io::{BufReader,BufRead};
use std::fs::File;
use std::convert::TryInto;

use crate::vector::Vector;
use super::Obj;
use crate::builtins::BUILTIN_LEVELS;
use crate::dgens::{contains_only};

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
#[derive(Clone,PartialEq,PartialOrd)]
pub struct Level {
	title: String,
	pub w: usize,
	pub h: usize,
	pub human_pos: Vector,
	noboxx_pts: Vec::<Vector>,
	boxx_pts: Vec::<Vector>,
	data: Vec::<Obj>,
}

#[derive(Clone,PartialEq,PartialOrd)]
pub struct SpLevel {		/* special level for solving */
	pub w: usize,
	pub human_pos: Vector,
	data: Vec::<Obj>,
}

impl SpLevel {
	pub fn new_from_level(level: &Level) -> Self {
		Self {
			w: level.w,
			human_pos: level.human_pos.clone(),
			data: level.data.clone(),
		}
	}
	pub fn get_obj_at_pt(&self, pt: &Vector) -> Obj {
		self.data[(pt.0 as usize) + (pt.1 as usize) * (self.w as usize)]
	}
	pub fn get_obj_at_idx(&self, idx: usize) -> Obj {
		self.data[idx]
	}
	pub fn set_obj_at_idx(&mut self, idx: usize, obj: Obj) {
		self.data[idx] = obj;
	}
	pub fn have_win_condition(&self) -> bool {
		for obj in self.data.iter() {
			match obj {
				Obj::Boulder | Obj::Hole | Obj::HumanInHole => return false,
				_ => {},
			};
		}
		return true;
	}
	pub fn eq_data(&self, b: &SpLevel) -> bool {
		self.data == b.data
	}	
}

#[wasm_bindgen]
impl Level {
	pub fn clone(&self) -> Level {
		Level {
			title: self.title.clone(),
			w: self.w,
			h: self.h,
			human_pos: self.human_pos.clone(),
			boxx_pts: self.boxx_pts.clone(),
			noboxx_pts: self.noboxx_pts.clone(),
			data: self.data.clone(),
		}
	}
	pub fn get_obj_at_pt(&self, pt: &Vector) -> Obj {
		self.data[(pt.0 as usize) + (pt.1 as usize) * (self.w as usize)]
	}
	pub fn get_obj_at_idx(&self, idx: usize) -> Obj {
		self.data[idx]
	}
	pub fn set_obj_at_idx(&mut self, idx: usize, obj: Obj) {
		self.data[idx] = obj;
	}
	pub fn get_data(&self) -> Array {
		self.data.clone().into_iter().map(|obj| JsValue::from(obj as u32)).collect()
	}
	fn get_vslice(&self, x: usize, y0: usize, y1: usize) -> Vec::<Obj> {
		let mut rv = Vec::<Obj>::new();
		for i in y0..y1 {
			rv.push(self.get_obj_at_pt(&Vector(x as i32,i as i32)));
		}
		return rv;
	}
	fn get_hslice(&self, x0: usize, x1: usize, y: usize) -> Vec::<Obj> {
		let mut rv = Vec::<Obj>::new();
		for i in x0..x1 {
			rv.push(self.get_obj_at_pt(&Vector(i as i32,y as i32)));
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
		return JsString::from(self.title.as_str());
	}
	pub fn have_win_condition(&self) -> bool {
		for obj in self.data.iter() {
			match obj {
				Obj::Boulder | Obj::Hole | Obj::HumanInHole => return false,
				_ => {},
			};
		}
		return true;
	}

}

// non-js
impl Level {
	pub fn do_noboxx_pts(&mut self) {
		let mut noboxx_pts: Vec::<Vector> = Vec::new();
		// aside from #, there are some points where box's simply can't go
		// e.g. the 2x2 [*#][# ] in any orientation (where space could be human too)

		let block_match = [Obj::Space,Obj::Wall,Obj::Wall];
		let block_match2 = [Obj::Human,Obj::Wall,Obj::Wall];
		for y in 0..self.h-1 {
			for x in 0..self.w-1 {
				for z in &X1VARS {
					let pt = Vector(x as i32,y as i32);
					let objs = [ self.get_obj_at_pt(&pt.add(&z[0])), 
								 self.get_obj_at_pt(&pt.add(&z[1])), 
								 self.get_obj_at_pt(&pt.add(&z[2])), ];
					if objs == block_match || objs == block_match2 {
						noboxx_pts.push(pt.add(&z[0]));
					} 
				}
			}
		} 

		self.noboxx_pts = noboxx_pts;
	
		// walls that follow the following pattern also can't have boxxes
		//  ##...# 
		// #      #

		let hall_start = vec![ vec![ Obj::Wall, Obj::Space, Obj::Space, Obj::Space ],
						       vec![ Obj::Wall, Obj::Human, Obj::Space, Obj::Space ],
						       vec![ Obj::Wall, Obj::Space, Obj::Human, Obj::Space ],
						       vec![ Obj::Wall, Obj::Space, Obj::Space, Obj::Human ] ];
		let hall_len = 4;

		struct HallInfoH {
			x: usize,
			y: usize,
			end_x: usize,
		};

		let mut start_x: Option<usize> = None;
		let mut halls = Vec::<HallInfoH>::new();

		// find the hall '    '+
		for y in 0..self.h {
			for x in 0..self.w {
				let obj_here = self.get_obj_at_idx(y * self.w + x);
				if start_x.is_some() && (obj_here == Obj::Space || obj_here == Obj::Human) { 	// Continuation of hallway
					// do nothing
				} else if start_x.is_some() && obj_here == Obj::Wall {				// We have end of the hall					
					halls.push( HallInfoH { x:start_x.unwrap(), y, end_x:x } );
					start_x = None;
				} else if start_x.is_some() {										// Not a real hallway
					start_x = None;
				}
				if start_x.is_none() && self.w >= 4 && x <= (self.w-hall_len) {
					if hall_start.contains(&self.get_hslice(x, x+hall_len, y)) {
						start_x = Some(x);
					}
				}
			}
		} 
		
		// check if the hall is a valid hall (has a complete wall on one side)
		for h in halls {
			let range1 = self.data.get(((h.y-1)*self.w+h.x+1)..((h.y-1)*self.w+h.end_x)).unwrap();		
			let range2 = self.data.get(((h.y+1)*self.w+h.x+1)..((h.y+1)*self.w+h.end_x)).unwrap();
			if contains_only(range1, &Obj::Wall) || contains_only(range2, &Obj::Wall) {
				(h.x+1..h.end_x).into_iter().for_each( |x| self.noboxx_pts.push(Vector(x as i32, h.y as i32)));
			}
		}

		// now do it all vertically!
		struct HallInfoV {
			x: usize,
			y: usize,
			end_y: usize,
		};
		let mut start_y: Option<usize> = None;
		let mut halls = Vec::<HallInfoV>::new();

		// find the hall '    '+
		for x in 0..self.w {
			for y in 0..self.h {
				let obj_here = self.get_obj_at_idx(y * self.w + x);
				if start_y.is_some() && (obj_here == Obj::Space || obj_here == Obj::Human) { 	// Continuation of hallway
					// do nothing
				} else if start_y.is_some() && obj_here == Obj::Wall {				// We have end of the hall					
					halls.push( HallInfoV { x:x, y:start_y.unwrap(), end_y:y } );
					start_y = None;
				} else if start_y.is_some() {										// Not a real hallway
					start_y = None;
				}
				if start_y.is_none() && self.h >= 4 && y <= (self.h-hall_len) {
					if hall_start.contains(&self.get_vslice(x, y, y+hall_len)) {		// Start of the hallway
						start_y = Some(y);
					}
				}
			}
		} 
		
		// check if the hall is a valid hall (has a complete wall on one side)
		for h in halls {
			let range1 = self.get_vslice(h.x-1, h.y+1, h.end_y);
			let range2 = self.get_vslice(h.x+1, h.y+1, h.end_y);
			if contains_only(&range1, &Obj::Wall) || contains_only(&range2, &Obj::Wall) {
				(h.y+1..h.end_y).into_iter().for_each( |y| self.noboxx_pts.push(Vector(h.x as i32, y as i32)));
			}
		}

		self.noboxx_pts.sort_unstable();
		self.noboxx_pts.dedup();
	}
	pub fn do_boxx_pts(&mut self) {
		let mut pts: Vec::<Vector> = Vec::new();
		for y in 0..self.h {
			for x in 0..self.w {
				let pt = Vector(x.try_into().unwrap(),y.try_into().unwrap());
				let obj = self.get_obj_at_pt(&pt);
				if obj == Obj::Boulder || obj == Obj::BoulderInHole {
					pts.push(pt);
				}
			}
		}
		self.boxx_pts = pts;
	}
	pub fn in_noboxx_pts(&self, v: Vector) -> bool {
		self.noboxx_pts.contains(&v)
	}
	pub fn strip_sprites(&mut self) {
		for idx in 0..(self.w * self.h) {
			let obj = self.get_obj_at_idx(idx);
			let nobj = match obj {
				Obj::Human => Obj::Space,
				Obj::HumanInHole => Obj::Hole,
				Obj::Boulder => Obj::Space,
				Obj::BoulderInHole => Obj::Hole,
				_ => obj,
			};
			self.set_obj_at_idx(idx,nobj);
		}		
	}
	pub fn eq_data(&self, b: &Level) -> bool {
		self.data == b.data
	}
	pub fn get_boxx_pts(&self) -> &Vec<Vector> {
		return &self.boxx_pts;
	}
}

pub fn load_level(filename: &str) -> Result<Level,String> {
    let input = match File::open(filename) {
		Ok(x) => x,
		_ => panic!("Failed to open level file: {}", filename),
	};
    let buffered = BufReader::new(input);
	let mut count: usize = 0;
	let mut w = 0;
	let mut data = Vec::<Obj>::with_capacity(128);
	let mut human_pos: Vector = Vector(0,0);

    for line in buffered.lines() {		
		let txt: String = match line {
			Ok(o) => o,
			_ => panic!("Failed to read line from level file."),
		};
        if count == 0 {
			// read in length
			w = txt.len();			
		}
		// check length equal to w
		if txt.len() == w {	
			// split line into characters
			for (i,c) in txt.char_indices() {		// chars() is iterator
				if c == '&' || c == '%' {
					// found human_pos
					human_pos = Vector(i.try_into().unwrap(),count.try_into().unwrap());
				}
				data.push( Obj::from_char(&c) );
			}
		} else {
			break;
		}
		
		count = count + 1;
    }
	let h  = count;

	println!("Dimensions: {} x {}", w, h);
	
	if human_pos.0 == 0 || human_pos.1 == 0 {
		panic!("Human not found in level");
	}
	
	println!("Human at: {}, {}", human_pos.0, human_pos.1);
	
	// my_vec.push(Wall);
	if w < 3 || h < 3 {
		panic!("Width and Height must be >= 3");
	}
	
	let mut level = Level {
		title: String::from("Untitled"),
		w: w,
		h: h,
		human_pos: human_pos,
		noboxx_pts: Vec::new(),
		boxx_pts: Vec::new(),
		data: data,
	};
	level.do_noboxx_pts();
	level.do_boxx_pts();
	return Ok(level);
}


#[wasm_bindgen]
pub fn load_builtin(number: usize) -> Option<Level> {
	// locate string
	if number >= BUILTIN_LEVELS.len() {
		return None;
	}
	
	let level = BUILTIN_LEVELS[number];
	
	// process string
	let mut count: usize = 0;
	let mut w = 0;
	let mut data = Vec::<Obj>::with_capacity(128);
	let mut human_pos: Vector = Vector(0,0);
	let mut boxx_pts = Vec::<Vector>::new();
	let mut level_title: String = String::from("Untitled");

    for line in level.lines() {		
		let txt = line.to_string();
		//println!("{} : {} : {}",txt,line, count);
		if count == 0 {
			// read in title of level
			level_title = txt;
			count += 1;
			continue;
		}
        if count == 1 {
			// read in width of level
			w = txt.len();			
		}
		// check length equal to w
		if txt.len() == w {	
			// split line into characters
			for (i,c) in txt.char_indices() {		// chars() is iterator
				if c == '&' || c == '%' {
					// found human_pos
					human_pos = Vector(i.try_into().unwrap(),(count-1).try_into().unwrap());
				} else if c == '*' || c == '@' {
					boxx_pts.push(Vector(i.try_into().unwrap(),(count-1).try_into().unwrap()));
				}
				data.push( Obj::from_char(&c) );
			}
		} else {
			break;
		}
		
		count = count + 1;
    }
	let h  = count - 1;

	println!("Dimensions: {} x {}", w, h);
	
	if human_pos.0 == 0 || human_pos.1 == 0 {
		panic!("Human not found in level");
	}
	
	println!("Human at: {}, {}", human_pos.0, human_pos.1);
	
	if w < 3 || h < 3 {
		panic!("Width and Height must be >= 3");
	}
	
	let mut level = Level {
		title: level_title,
		w: w,
		h: h,
		noboxx_pts: Vec::new(),
		boxx_pts: boxx_pts,
		human_pos: human_pos,
		data: data,
	};
	level.do_boxx_pts();
	level.do_noboxx_pts();
	
	return Some(level);
}
