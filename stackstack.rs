// Box Bopper: Sokoban clone in rust
// Copyright David Atkinson 2020-2021
//
// stackstack.rs: a stack on the stack, used to speed up inner loops by avoiding memory allocation

const STACKSTACK64_MAX: usize = 8;

#[derive(Copy,Clone)]
pub struct StackStack64 {
	pub next: usize,
	pub stack: [u64; STACKSTACK64_MAX],
}

impl StackStack64 {
	pub fn new() -> StackStack64 {
		StackStack64 {
			next: 0,
			//stack: [0; STACKSTACK64_MAX],
			stack: unsafe { std::mem::MaybeUninit::uninit().assume_init() },
		}
	}
	pub fn push(&mut self, d: u64) {
        if self.next == STACKSTACK64_MAX { panic!("StackStack64 overflow"); }
        self.stack[self.next] = d;
		self.next += 1;
	}
	pub fn pop(&mut self) -> u64 {
        if self.next == 0 { panic!("StackStack64 underflow"); }
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

const STACKSTACK32_MAX: usize = 64;

#[derive(Copy,Clone)]
pub struct StackStack32 {
	pub next: usize,
	pub stack: [u32; STACKSTACK32_MAX],
}

impl StackStack32 {
	pub fn new() -> StackStack32 {
		StackStack32 {
			next: 0,
			//stack: [0; STACKSTACK32_MAX],
			stack: unsafe { std::mem::MaybeUninit::uninit().assume_init() },
		}
	}
	pub fn push(&mut self, d: u32) {
        if self.next == STACKSTACK32_MAX { panic!("StackStack32 overflow"); }
        self.stack[self.next] = d;
		self.next += 1;
	}
	pub fn pop(&mut self) -> u32 {
        if self.next == 0 { panic!("StackStack32 underflow"); }
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

const STACKSTACK16_MAX: usize = 64;

#[derive(Copy,Clone)]
pub struct StackStack16 {
	pub next: usize,
	pub stack: [u16; STACKSTACK16_MAX],
}

impl StackStack16 {
	pub fn new() -> StackStack16 {
		StackStack16 {
			next: 0,
			//stack: [0; STACKSTACK16_MAX],
			stack: unsafe { std::mem::MaybeUninit::uninit().assume_init() },
		}
	}
	pub fn push(&mut self, d: u16) {
		if self.next == STACKSTACK16_MAX { panic!("StackStack16 overflow"); }
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

// We can implement Clone clone_from for StackStack16, and get the raw ptr to copy the minimal number of bytes

const STACKSTACK8_MAX: usize = 64;

#[derive(Copy,Clone)]
pub struct StackStack8 {
	pub next: usize,
	pub stack: [u8; STACKSTACK8_MAX],
}

impl StackStack8 {
	pub fn new() -> StackStack8 {
		StackStack8 {
			next: 0,
			stack: unsafe { std::mem::MaybeUninit::uninit().assume_init() },
		}
	}
	pub fn push(&mut self, d: u8) {
		if self.next == STACKSTACK8_MAX { panic!("StackStack8 overflow"); }
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