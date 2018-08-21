// RandVecIter is an iterator that returns a random value from a RandVec.
use rand::Rng;

pub struct RandVec<T>
{
	vec: Vec<T>
}

impl<T: Clone> RandVec<T>
{
	pub fn new(vec: Vec<T>) -> Self
	{
		Self { vec: vec }
	}

	pub fn rand_iter<'a, R: Rng>(&'a self, rng: R) -> RandVecIter<T, R>
	{
		RandVecIter{ vec: self, rng: rng }
	}

	fn next_rand<'a, R: Rng>(&'a self, rng: &mut R) -> Option<T>
	{
		// generate a random index and return it
		let index = rng.gen_range(0, self.vec.len());
		Some(self.vec[index].clone())
	}
}

pub struct RandVecIter<'a, T: Clone + 'a, R: Rng + 'a>
{
	vec: &'a RandVec<T>,
	rng: R
}

impl<'a, T: Clone + 'a, R: Rng> Iterator for RandVecIter<'a, T, R>
{
	type Item = T;

	fn next(&mut self) -> Option<T>
	{
		self.vec.next_rand(&mut self.rng)
	}
}