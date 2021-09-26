// Box Bopper: Sokoban-like game
// Copyright David Atkinson 2020-2021
//
// sprite.rs: Sprite animation / management for boxbopper

use wasm_bindgen::prelude::*;

use crate::vector::Vector;
use super::{Obj,console_log};
use crate::time::{get_time_ms};

#[wasm_bindgen]
pub fn tween(x: f64) -> f64 {
	// input between 0 and 1 (or 0 and -1 with alternate function)
	// returns tween between 0 and 1 that is slow at beginning and end, fast in middle
	// basic tween is y = Ax + (1-A)(0.5(-cos(pi*x)+1)) -- where A is linear component (vs sinusoidal component)
	let lin_component: f64 = 0.5;
	if x < -1.0 { return -1.0; }
	if x > 1.0  { return 1.0;  }
	if x < 0.0 { // for 0..-1
		return lin_component * x + (1.0 - lin_component) * (0.5 * (f64::cos(std::f64::consts::PI * x) - 1.0));	
	} // for 0..+1
	return lin_component * x + (1.0 - lin_component) * (0.5 * (- f64::cos(std::f64::consts::PI * x) + 1.0));
}

#[wasm_bindgen]
#[derive(Clone,Copy)]
pub struct SpriteInfo {		// location information passed back to JS so it can render the sprite in the correct location
	pub id: u32,
	pub obj: Obj,
	pub x: f64,
	pub y: f64,
}

#[wasm_bindgen]
#[derive(Clone,Copy)]
pub struct Trans {
	pub initial_xy: Vector,
	pub final_xy: Vector,
	pub duration: f64,
}

#[wasm_bindgen]
#[derive(Clone)]
pub struct Sprite {
	pub id: u32,	// unique id for objects of this Obj type. i.e. playerNumber or boulderNumber.
	pub obj: Obj,	// what type of object affects how we render it
	pub initial_xy: Vector,		// movement transition to apply
	pub initial_time: f64,
	pub final_xy: Vector,
	pub duration: f64,
	priv_is_moving: u32,
}

impl Sprite {
	pub fn new(id: u32, obj: Obj, initial_time: f64, duration: f64, initial_xy: Vector, final_xy: Vector) -> Sprite {
		Sprite {
			id,
			obj,
			initial_time,
			duration,
			initial_xy,
			final_xy,
			priv_is_moving: 0,
		}
	}
	pub fn apply_trans(&mut self, trans: Trans) {		
		if !self.is_moving() { 
			self.initial_time = get_time_ms();
			self.duration = trans.duration;
			self.initial_xy = trans.initial_xy.clone();
			self.final_xy = trans.final_xy.clone();
		} else {
			// ignore the requested movement !
			console_log("move requested while already moving!");
		}
	}
	pub fn is_moving(&self) -> bool {
		get_time_ms() < (self.initial_time + self.duration)
	}

	pub fn get_xy(&mut self) -> [f64;2] {
		// linear
		let t = get_time_ms();
		if t <= self.initial_time {
			return [f64::from(self.initial_xy.0), f64::from(self.initial_xy.1)];
		} 
		if t >= (self.initial_time + self.duration) {
			// update location - don't do this here, it doesn't work
			//self.initial_xy = self.final_xy.clone();
			//console_log("post movement 1");

			return [self.final_xy.0.into(), self.final_xy.1.into()];
		}
		// according to time & duration, we are currently moving
		let mut delta: f64 = (t - self.initial_time) / self.duration;
		if delta > 1_f64 {
			delta = 1_f64;
			self.initial_xy = self.final_xy.clone();
			console_log("post movement 2");
		}
		// linear:
		// let nx = delta * f64::from(self.final_xy.0 - self.initial_xy.0) + f64::from(self.initial_xy.0);
		// let ny = delta * f64::from(self.final_xy.1 - self.initial_xy.1) + f64::from(self.initial_xy.1);
		// tween:
		let nx = tween(delta) * f64::from(self.final_xy.0 - self.initial_xy.0) + f64::from(self.initial_xy.0);
		let ny = tween(delta) * f64::from(self.final_xy.1 - self.initial_xy.1) + f64::from(self.initial_xy.1);
		[nx,ny]
	}
	pub fn get_sprite_info(&mut self) -> SpriteInfo {
		let pt = self.get_xy();
		SpriteInfo {
			id: self.id,
			obj: self.obj,
			x: pt[0],
			y: pt[1],
		}
	}
}
