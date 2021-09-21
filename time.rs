// Box Bopper: Sokoban clone in rust
// Copyright David Atkinson 2020-2021
//
// time.rs: basic get_time_ms for boxbopper

#[cfg(not(target_arch = "wasm32"))]
use std::time::{SystemTime,UNIX_EPOCH};

// we need time in msec since unix epoch (for js compatibility)
#[cfg(not(target_arch = "wasm32"))]
pub fn get_time_ms() -> f64 {
    let since_the_epoch = SystemTime::now()
        .duration_since(UNIX_EPOCH)
		.expect("Time went backwards");
	let ms: u64 = since_the_epoch.as_secs() * 1000 + since_the_epoch.subsec_nanos() as u64 / 1_000_000;
	let lopart: u32 = (ms & 0xFFFFFFFF) as u32; 
	let hipart: u32 = (ms >> 32) as u32;
	let t: f64 = f64::from(lopart) + (f64::from(hipart) * 4.294967296e9);
	t
}

#[cfg(target_arch = "wasm32")]
pub fn get_time_ms() -> f64 {
	let t = (js_sys::Date::now() as u64 / 10) * 10;
	return t as f64;
}
