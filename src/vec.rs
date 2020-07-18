use replace_with::on_return_or_unwind;
use std::{alloc::Layout, marker::PhantomData, mem::ManuallyDrop};

use super::{r#try, Try};

/// A type that contains useful meta-data about a
/// the Vec<_> that it was created from
struct Input<T> {
	// the start of the vec data segment
	start: *mut T,

	// the current position in the vec data segment
	ptr: *mut T,

	// the length of the vec data segment
	len: usize,

	// the capacity of the vec data segment
	cap: usize,

	drop: PhantomData<T>,
}

impl<T> From<Vec<T>> for Input<T> {
	fn from(vec: Vec<T>) -> Self {
		let mut vec = ManuallyDrop::new(vec);

		let ptr = vec.as_mut_ptr();

		Self {
			start: ptr,
			ptr,
			len: vec.len(),
			cap: vec.capacity(),
			drop: PhantomData,
		}
	}
}

/// Extension methods for `Vec<T>`
pub trait VecExt: Sized {
	/// The type that the `Vec<T>` stores
	type T;

	/// Map a vector to another vector, will try and reuse the allocation if the
	/// allocation layouts of the two types match, i.e. if
	/// `std::alloc::Layout::<T>::new() == std::alloc::Layout::<U>::new()`
	/// then the allocation will be reused
	fn map<U, F: FnMut(Self::T) -> U>(self, mut f: F) -> Vec<U> {
		use std::convert::Infallible;

		match self.try_map(move |x| Ok::<_, Infallible>(f(x))) {
			Ok(x) => x,
			Err(x) => match x {},
		}
	}

	/// Map a vector to another vector, will try and reuse the allocation if the
	/// allocation layouts of the two types match, i.e. if
	/// `std::alloc::Layout::<T>::new() == std::alloc::Layout::<U>::new()`
	/// then the allocation will be reused
	///
	/// The mapping function can be fallible, and on early return, it will drop all previous values,
	/// and the rest of the input vector. Thre error will be returned as a `Result`
	fn try_map<U, R: Try<Ok = U>, F: FnMut(Self::T) -> R>(self, f: F) -> Result<Vec<U>, R::Error>;

	/// Drops all of the values in the vector and
	/// create a new vector from it if the layouts are compatible
	///
	/// if layouts are not compatible, then return `Vec::new()`
	fn recycle<U>(self) -> Vec<U>;
}

impl<T> VecExt for Vec<T> {
	type T = T;

	fn try_map<U, R: Try<Ok = U>, F: FnMut(Self::T) -> R>(self, f: F) -> Result<Vec<U>, R::Error> {
		// try_zip_with! { self => |x| { f(x) } }

		if Layout::new::<T>() == Layout::new::<U>() {
			let iter = MapIter {
				init_len: 0,
				data: Input::from(self),
				drop: PhantomData,
			};

			iter.try_into_vec(f)
		} else {
			self.into_iter().map(f).map(R::into_result).collect()
		}
	}

	fn recycle<U>(mut self) -> Vec<U> {
		self.clear();

		// no more elements in the vector
		self.map(|_| unsafe { std::hint::unreachable_unchecked() })
	}
}

struct MapIter<T, U> {
	init_len: usize,

	data: Input<T>,

	// for drop check
	drop: PhantomData<U>,
}

impl<T, U> MapIter<T, U> {
	fn try_into_vec<R: Try<Ok = U>, F: FnMut(T) -> R>(
		mut self, mut f: F,
	) -> Result<Vec<U>, R::Error> {
		// does a pointer walk, easy for LLVM to optimize
		while self.init_len < self.data.len {
			unsafe {
				let value = r#try!(f(self.data.ptr.read()));

				(self.data.ptr as *mut U).write(value);

				self.data.ptr = self.data.ptr.add(1);
				self.init_len += 1;
			}
		}

		let vec = ManuallyDrop::new(self);

		// we don't want to free the memory
		// which is what dropping this `MapIter` will do
		unsafe {
			Ok(Vec::from_raw_parts(
				vec.data.start as *mut U,
				vec.data.len,
				vec.data.cap,
			))
		}
	}
}

impl<T, U> Drop for MapIter<T, U> {
	fn drop(&mut self) {
		unsafe {
			on_return_or_unwind(
				|| {
					// offset by 1 because self.ptr is pointing to
					// memory that was just read from, dropping that
					// would lead to a double free
					std::ptr::drop_in_place(std::slice::from_raw_parts_mut(
						self.data.ptr.add(1),
						self.data.len - self.init_len - 1,
					));
				},
				|| {
					// destroy the initialized output
					let _ = Vec::from_raw_parts(
						self.data.start as *mut U,
						self.init_len,
						self.data.cap,
					);
				},
			)
		}
	}
}
