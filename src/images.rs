// ImageManager handles reading from directories and returning random images.

extern crate glob;
extern crate rand;

use self::glob::glob;
use rand_iter::{RandVec, RandVecIter};
use std::error::Error;
use rand::prelude::*;
use std::iter::Take;
use std::collections::HashMap;

pub struct ImageManager
{
	images: RandVec<String>
}

impl ImageManager
{
	// Creates a new ImageManager using files found with the given glob string.
	pub fn new(glob_str: &str) -> Result<Self, Box<Error>>
	{
		let files = try!(glob(glob_str)).filter_map(Result::ok);
		let filenames : Vec<String> = files
			.filter(|p| p.is_file())
			.filter_map(|p| {
				let of = p.file_name();
				match of {
					Some(f) => Some(f.to_string_lossy().into_owned()),
					None => None
				}
			})
			.collect();

		let vec = RandVec::new(filenames);

		Ok(Self { images: vec })
	}

	pub fn get_rand_iter(&self, num: usize) -> Take<RandVecIter<String, rand::ThreadRng>>
	{
		self.images.rand_iter(thread_rng()).take(num)
	}
}

pub struct Endpoints<'a>
{
	endpoints: HashMap<&'a str, ImageManager>
}

impl<'a> Endpoints<'a>
{
	pub fn new() -> Self
	{
		Self { endpoints: HashMap::new() }
	}

	pub fn add(&mut self, key: &'static str, glob: &'static str) -> Result<(), Box<Error>>
	{
		let im = try!(ImageManager::new(glob));
		self.endpoints.insert(key, im);
		Ok(())
	}

	pub fn get(&'a self, name: &str) -> Option<&'a ImageManager>
	{
		match self.endpoints.get(name) {
			Some(im) => Some(&im),
			None => None
		}
	}
}