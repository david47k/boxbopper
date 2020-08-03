
// generic functions that are useful

pub fn contains_only<T>(a: &[T], b: &T) -> bool where T: Eq {
	for item in a {	
		if item != b {
			return false;
		}
	}
	return true;
} 

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
