
// generic functions that are useful

// this looks like it's just a.all(|x| x==b)
pub fn contains_only<T>(a: &[T], b: &T) -> bool where T: Eq {
	for item in a {	
		if item != b {
			return false;
		}
	}
	return true;
} 

// verifies a contains only items in the set b
pub fn contains_only_set<T>(a: &[T], b: &[T]) -> bool where T: Eq {
	'outer: for item in a {	
		for item2 in b {
			if item == item2 {
				continue 'outer;
			}
		}
		return false;
	}
	return true;
}
