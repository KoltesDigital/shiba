use nom::{
	error::{ErrorKind, ParseError},
	Err, IResult, InputIter, InputLength, Offset, Slice,
};
use std::cell::Cell;
use std::ops::{RangeFrom, RangeTo};

/// Used for getting the parsed position as pointer.
pub trait InputSliceAsPtr {
	fn as_ptr(&self) -> *const ();
}

impl<'a, T> InputSliceAsPtr for &'a [T] {
	#[inline]
	fn as_ptr(&self) -> *const () {
		<[T]>::as_ptr(self) as *const ()
	}
}

impl<'a> InputSliceAsPtr for &'a str {
	#[inline]
	fn as_ptr(&self) -> *const () {
		<str>::as_ptr(self) as *const ()
	}
}

/// Succeeds only when the input is at the beginning.
pub fn begin<I: InputSliceAsPtr, E: ParseError<I>>(all: I) -> impl Fn(I) -> IResult<I, (), E> {
	move |i: I| {
		if all.as_ptr() == i.as_ptr() {
			Ok((i, ()))
		} else {
			Err(Err::Error(E::from_error_kind(i, ErrorKind::Eof)))
		}
	}
}

/// Succeeds only when the input is at the end.
pub fn end<I: InputIter, E: ParseError<I>>() -> impl Fn(I) -> IResult<I, (), E> {
	move |i: I| match i.iter_elements().next() {
		Some(_) => Err(Err::Error(E::from_error_kind(i, ErrorKind::Eof))),
		_ => Ok((i, ())),
	}
}

/// Succeeds only once.
pub fn once<I, E: ParseError<I>>() -> impl Fn(I) -> IResult<I, (), E> {
	let first = Cell::new(true);
	move |i: I| {
		if first.get() {
			first.set(false);
			Ok((i, ()))
		} else {
			Err(Err::Error(E::from_error_kind(i, ErrorKind::Eof)))
		}
	}
}

/// Succeeds only when both children parser succeed.
pub fn take_unless<
	I: Clone + InputLength + Offset + Slice<RangeFrom<usize>> + Slice<RangeTo<usize>>,
	O,
	E: ParseError<I>,
	F: Fn(I) -> IResult<I, Option<O>, E>,
>(
	parser: F,
) -> impl Fn(I) -> IResult<I, (I, O), E> {
	move |i: I| {
		let mut it = i.clone();
		let mut count = 0;
		loop {
			match parser(it.clone()) {
				Ok((i2, o2)) => match o2 {
					Some(o2) => {
						break Ok((i2, (i.slice(..count), o2)));
					}
					None => {
						count += it.offset(&i2);
						it = i2;
					}
				},
				Err(_) => {
					if it.input_len() == 0 {
						break Err(Err::Error(E::from_error_kind(it, ErrorKind::Eof)));
					} else {
						count += 1;
						it = it.slice(1..);
					}
				}
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use nom::{bytes::complete::*, combinator::*, multi::*, sequence::*};

	#[test]
	fn test_begin() {
		let parser = |s| many0::<_, _, (), _>(preceded(begin(s), tag("a")))(s);
		assert_eq!(parser("a"), Ok(("", vec!["a"])));
		assert_eq!(parser("aa"), Ok(("a", vec!["a"])));
	}

	#[test]
	fn test_end() {
		let parser = many0::<_, _, (), _>(terminated(tag("a"), end()));
		assert_eq!(parser("a"), Ok(("", vec!["a"])));
		assert_eq!(parser("aa"), Ok(("aa", vec![])));
	}

	#[test]
	fn test_once() {
		let parser = || many0::<_, _, (), _>(preceded(once(), tag("a")));
		assert_eq!(parser()("a"), Ok(("", vec!["a"])));
		assert_eq!(parser()("aa"), Ok(("a", vec!["a"])));
	}

	#[test]
	fn test_take_unless() {
		let parser = many0::<_, _, (_, _), _>(take_unless(map(tag(","), Some)));
		assert_eq!(parser("a"), Ok(("a", vec![])));
		assert_eq!(parser("ab,cd"), Ok(("cd", vec![("ab", ",")])));
	}
}
