use super::*;
use crate::types::{Variable, VariableKind};
use nom::{
	branch::*, bytes::complete::*, character::complete::*, combinator::*, multi::*, sequence::*,
	IResult,
};
use std::slice;
use std::str;

pub fn identifier<'a>(input: &'a str) -> IResult<&'a str, &'a str> {
	map(
		tuple((
			take_while_m_n(1, 1, |c: char| c == '_' || c.is_alphabetic()),
			take_while(|c: char| c == '_' || c.is_alphanumeric()),
		)),
		|(first, rest): (&str, &str)| {
			let ptr = first.as_ptr();
			unsafe {
				let slice = slice::from_raw_parts(ptr, rest.len() + 1);
				str::from_utf8(slice)
			}
			.unwrap()
		},
	)(input)
}

pub fn identifier_length(input: &str) -> IResult<&str, (&str, Option<usize>)> {
	map(
		tuple((
			identifier,
			space0,
			opt(delimited(char('['), parse_digit1, char(']'))),
		)),
		|(identifier, _, length)| (identifier, length),
	)(input)
}

pub fn identifier_length_value(input: &str) -> IResult<&str, (&str, Option<usize>, &str)> {
	map(
		tuple((
			identifier_length,
			space0,
			char('='),
			space0,
			take_while(|c: char| c != ';' && c != ','),
		)),
		|((identifier, length), _, _, _, value)| (identifier, length, value.trim()),
	)(input)
}

pub fn const_variables(input: &str) -> IResult<&str, Vec<Variable>> {
	map(
		tuple((
			tag("const"),
			space1,
			identifier,
			space1,
			separated_list(tuple((char(','), space0)), identifier_length_value),
			char(';'),
		)),
		|(_, _, type_name, _, list, _)| {
			list.into_iter()
				.map(|(name, length, value)| Variable {
					active: true,
					kind: VariableKind::Const(value.to_string()),
					length,
					minified_name: None,
					name: name.to_string(),
					type_name: type_name.to_string(),
				})
				.collect()
		},
	)(input)
}

pub fn regular_variables(input: &str) -> IResult<&str, Vec<Variable>> {
	map(
		tuple((
			identifier,
			space1,
			separated_list(tuple((char(','), space0)), identifier_length),
			char(';'),
		)),
		|(type_name, _, list, _)| {
			list.into_iter()
				.map(|(name, length)| Variable {
					active: true,
					kind: VariableKind::Regular,
					length,
					minified_name: None,
					name: name.to_string(),
					type_name: type_name.to_string(),
				})
				.collect()
		},
	)(input)
}

pub fn uniform_variables(input: &str) -> IResult<&str, Vec<Variable>> {
	map(
		tuple((
			tag("uniform"),
			space1,
			identifier,
			space1,
			separated_list(tuple((char(','), space0)), identifier_length),
			char(';'),
		)),
		|(_, _, type_name, _, list, _)| {
			list.into_iter()
				.map(|(name, length)| Variable {
					active: true,
					kind: VariableKind::Uniform,
					length,
					minified_name: None,
					name: name.to_string(),
					type_name: type_name.to_string(),
				})
				.collect()
		},
	)(input)
}

pub fn variables(input: &str) -> IResult<&str, Vec<Variable>> {
	map(
		many0(take_unless(alt((
			value(
				None,
				tuple((
					tag("precision"),
					space1,
					identifier,
					space1,
					identifier,
					space0,
					char(';'),
				)),
			),
			map(const_variables, Some),
			map(regular_variables, Some),
			map(uniform_variables, Some),
		)))),
		|variables_list| {
			variables_list
				.into_iter()
				.map(|(_, var)| var)
				.flatten()
				.collect()
		},
	)(input)
}

pub fn directive<'a, O, F: Fn(&'a str) -> IResult<&'a str, O>>(
	content: F,
) -> impl Fn(&'a str) -> IResult<&'a str, O> {
	move |input: &str| {
		let (input, (_, _, _, _, output, _)) = tuple((
			tag("#pragma"),
			space1,
			tag("shiba"),
			space1,
			&content,
			line_ending,
		))(input)?;
		Ok((input, output))
	}
}

fn parse_digit1(input: &str) -> IResult<&str, usize> {
	map_res(digit1, |index: &str| {
		index
			.parse::<usize>()
			.map_err(|_| Err::Failure((input, ErrorKind::Digit)))
	})(input)
}

pub fn fragment_directive(input: &str) -> IResult<&str, usize> {
	map(
		tuple((tag("fragment"), space1, parse_digit1)),
		|(_, _, index)| index,
	)(input)
}

pub fn vertex_directive(input: &str) -> IResult<&str, usize> {
	map(
		tuple((tag("vertex"), space1, parse_digit1)),
		|(_, _, index)| index,
	)(input)
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_identifier() {
		assert_eq!(identifier("uniformVar0"), Ok(("", "uniformVar0")));
	}

	#[test]
	fn test_variables() {
		let variables = variables(
			r#"precision mediump float;
float regularVar0;
float regularVar1[1];
#define foo bar
const float constVar0 = 42., constVar1 = 1337.;
uniform float uniformVar0;
uniform float uniformVar1[4];
uniform vec2 uniformVar2;
"#,
		);

		assert_eq!(
			variables,
			Ok((
				"\n",
				vec![
					Variable {
						active: true,
						kind: VariableKind::Regular,
						length: None,
						minified_name: None,
						name: "regularVar0".to_string(),
						type_name: "float".to_string(),
					},
					Variable {
						active: true,
						kind: VariableKind::Regular,
						length: Some(1),
						minified_name: None,
						name: "regularVar1".to_string(),
						type_name: "float".to_string(),
					},
					Variable {
						active: true,
						kind: VariableKind::Const("42.".to_string()),
						length: None,
						minified_name: None,
						name: "constVar0".to_string(),
						type_name: "float".to_string(),
					},
					Variable {
						active: true,
						kind: VariableKind::Const("1337.".to_string()),
						length: None,
						minified_name: None,
						name: "constVar1".to_string(),
						type_name: "float".to_string(),
					},
					Variable {
						active: true,
						kind: VariableKind::Uniform,
						length: None,
						minified_name: None,
						name: "uniformVar0".to_string(),
						type_name: "float".to_string(),
					},
					Variable {
						active: true,
						kind: VariableKind::Uniform,
						length: Some(4),
						minified_name: None,
						name: "uniformVar1".to_string(),
						type_name: "float".to_string(),
					},
					Variable {
						active: true,
						kind: VariableKind::Uniform,
						length: None,
						minified_name: None,
						name: "uniformVar2".to_string(),
						type_name: "vec2".to_string(),
					}
				]
			))
		);
	}

	#[test]
	fn test_directive() {
		let contents = take_unless(map(directive(alpha1), Some))(
			r#"precision mediump float;
float regularVar0;
#pragma shiba test
const float constVar = 42.;
"#,
		);

		assert_eq!(
			contents,
			Ok((
				"const float constVar = 42.;\n",
				("precision mediump float;\nfloat regularVar0;\n", "test")
			))
		);
	}
}
