

use wasm_bindgen::prelude::*;
use js_sys::{Array,JsString};

use std::io::{BufReader,BufRead};
use std::fs::File;
use std::convert::TryInto;

use crate::vector::Vector;
use super::Obj;
use crate::builtins::BUILTIN_LEVELS;


#[wasm_bindgen]
#[derive(Clone)]
pub struct Level {
	title: String,
	pub w: usize,
	pub h: usize,
	pub human_pos: Vector,
	data: Vec::<Obj>,
}

#[wasm_bindgen]
impl Level {
	pub fn clone(&self) -> Level {
		Level {
			title: self.title.clone(),
			w: self.w,
			h: self.h,
			human_pos: self.human_pos.clone(),
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
				if c == '&' || c == '@' {
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
	
	let level = Level {
		title: String::from("Untitled"),
		w: w,
		h: h,
		human_pos: human_pos,
		data: data,
	};
	
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
	
	let level = Level {
		title: level_title,
		w: w,
		h: h,
		human_pos: human_pos,
		data: data,
	};
	
	return Some(level);
}
