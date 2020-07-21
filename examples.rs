// This demonstrates using a newtype (struct wrapper around a type) to implement a function on a Vec::<unit> with deref

struct Moves (Vec::<Move>);

use std::ops::Deref;

impl Deref for Moves {
	type Target = Vec::<Move>;
    fn deref(&self) -> &Vec::<Move> {
        &self.0
    }
}

impl Moves {
	fn to_string(&self) -> String {
		let mut s: String = String::new();
		for m in self.0.iter() {
			s = s + match m {
				Move::North => "n",
				Move::East =>  "e",
				Move::South => "s",
				Move::West =>  "w",
			};
		}
		return s;
	}	
}


// This demonstrates implementation of a Display trait for a custom type

impl std::fmt::Display for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
			Move::North => write!(f, "n"),
			Move::East  => write!(f, "e"),
			Move::South => write!(f, "s"),
			Move::West  => write!(f, "w"),
		}
    }
}