// Box Bopper: Sokoban-like game
// Copyright David Atkinson 2020-2021
//
// stackstack.rs: a stack on the stack, used to speed up inner loops by avoiding memory allocation
//
// default multiplier is 1 (512 bits or path of 256) but it'll overflow with levels which can have long paths

const HARD_MAX_PATH: usize = 512;
const STACKMAX: usize = HARD_MAX_PATH * 2 / 128;		// 2 bits per move, 128 bit blocks

#[derive(Copy,Clone)]
pub struct StackStack<T: Copy> {
	pub next: usize,
	pub stack: [T; STACKMAX],
}

impl<T: Copy> StackStack<T> {
	pub fn new() -> Self {
		Self {
			next: 0,
			stack: unsafe { std::mem::MaybeUninit::uninit().assume_init() },
		}
	}
	pub fn push(&mut self, d: T) {
        if self.next == self.stack.len() { panic!("StackStack overflow"); }
        self.stack[self.next] = d;
		self.next += 1;
	}
	pub fn pop(&mut self) -> T {
        if self.next == 0 { panic!("StackStack underflow"); }
		self.next -= 1;
		self.stack[self.next]
	}
	pub fn len(&self) -> usize {
		self.next
	}
	pub fn clear(&mut self) {
		self.next = 0;
	}
}

// Used to store tail nodes when solving and unsolving levels
#[derive(Copy,Clone)]
pub struct StackStack16x64 {
	pub next: usize,
	pub stack: [u16; 64],
}

impl StackStack16x64 {
	pub fn new() -> Self {
		Self {
			next: 0,
			stack: unsafe { std::mem::MaybeUninit::uninit().assume_init() },
		}
	}
	pub fn push(&mut self, d: u16) {
		if self.next == self.stack.len() { panic!("StackStack16 overflow"); }
		self.stack[self.next] = d;
		self.next += 1;
	}
	pub fn pop(&mut self) -> u16 {
        if self.next == 0 { panic!("StackStack16 underflow"); }
		self.next -= 1;
		self.stack[self.next]
	}
	pub fn len(&self) -> usize {
		self.next
	}
	pub fn clear(&mut self) {
		self.next = 0;
	}
}

// Used when backtracing moves
#[derive(Copy,Clone)]
pub struct StackStack8x64 {
	pub next: usize,
	pub stack: [u8; 64],
}

impl StackStack8x64 {
	pub fn new() -> Self {
		Self {
			next: 0,
			stack: unsafe { std::mem::MaybeUninit::uninit().assume_init() },
		}
	}
	pub fn push(&mut self, d: u8) {
		if self.next == self.stack.len() { panic!("StackStack8 overflow. Path > 256?"); }
		self.stack[self.next] = d;
		self.next += 1;
	}
	pub fn pop(&mut self) -> u8 {
        if self.next == 0 { panic!("StackStack8 underflow"); }
        self.next -= 1;
		self.stack[self.next]
	}
	pub fn len(&self) -> usize {
		self.next
	}
	pub fn clear(&mut self) {
		self.next = 0;
	}
	pub fn reverse(&mut self) {
		let mut top = self.next - 1;
		for i in 0..self.next/2 {
			let swapper = self.stack[i];
			self.stack[i] = self.stack[top];
			self.stack[top] = swapper;
			top -= 1;
		}
	}
}