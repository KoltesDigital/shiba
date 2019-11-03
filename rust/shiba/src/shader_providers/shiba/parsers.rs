use super::*;
use crate::parsers::*;
use crate::types::{Variable, VariableKind};
use nom::{
	branch::*, bytes::complete::*, character::complete::*, combinator::*, error::ErrorKind,
	multi::*, sequence::*, Err, IResult,
};

fn fragment_pragma(input: &str) -> IResult<&str, Section> {
	let (input, _) = tag("fragment")(input)?;
	let (input, _) = space1(input)?;
	let (input, index) = digit1(input)?;
	let index = index
		.parse::<usize>()
		.map_err(|_| Err::Failure((input, ErrorKind::Digit)))?;
	Ok((input, Section::Fragment(index)))
}

fn vertex_pragma(input: &str) -> IResult<&str, Section> {
	let (input, _) = tag("vertex")(input)?;
	let (input, _) = space1(input)?;
	let (input, index) = digit1(input)?;
	let index = index
		.parse::<usize>()
		.map_err(|_| Err::Failure((input, ErrorKind::Digit)))?;
	Ok((input, Section::Vertex(index)))
}

fn section(input: &str) -> IResult<&str, Section> {
	let (input, _) = tag("#pragma")(input)?;
	let (input, _) = space1(input)?;
	let (input, _) = tag("shiba")(input)?;
	let (input, _) = space1(input)?;
	let (input, section) = alt((
		|input| tag("attributes")(input).and_then(|(input, _)| Ok((input, Section::Attributes))),
		|input| tag("common")(input).and_then(|(input, _)| Ok((input, Section::Common))),
		fragment_pragma,
		|input| tag("outputs")(input).and_then(|(input, _)| Ok((input, Section::Outputs))),
		|input| tag("varyings")(input).and_then(|(input, _)| Ok((input, Section::Varyings))),
		vertex_pragma,
	))(input)?;
	let (input, _) = line_ending(input)?;
	Ok((input, section))
}

fn sections(input: &str) -> IResult<&str, Vec<(&str, Section)>> {
	many0(take_unless(map(section, Some)))(input)
}

fn version(input: &str) -> IResult<&str, &str> {
	let (input, _) = once()(input)?;
	let (input, _) = tag("#version")(input)?;
	let (input, _) = space1(input)?;
	let (input, version) = not_line_ending(input)?;
	let (input, _) = line_ending(input)?;
	Ok((input, version))
}

pub type Contents<'a> = (Option<&'a str>, Vec<(&'a str, Section)>);

pub fn contents(input: &str) -> IResult<&str, Contents> {
	tuple((opt(version), sections))(input)
}

fn identifier(input: &str) -> IResult<&str, String> {
	let (input, tuple) = tuple((
		take_while_m_n(1, 1, |c: char| c == '_' || c.is_alphabetic()),
		take_while(|c: char| c == '_' || c.is_alphanumeric()),
	))(input)?;
	Ok((input, format!("{}{}", tuple.0, tuple.1)))
}

fn const_variable(input: &str) -> IResult<&str, Variable> {
	let (input, _) = tag("const")(input)?;
	let (input, _) = space1(input)?;
	let (input, type_name) = identifier(input)?;
	let (input, _) = space1(input)?;
	let (input, name) = identifier(input)?;
	let (input, _) = space0(input)?;
	let (input, _) = char('=')(input)?;
	let (input, _) = space0(input)?;
	let (input, value) = terminated(take_while(|c: char| c != ';'), char(';'))(input)?;
	Ok((
		input,
		Variable {
			active: true,
			kind: VariableKind::Const(value.to_string()),
			minified_name: None,
			name,
			type_name,
		},
	))
}

fn regular_variable(input: &str) -> IResult<&str, Variable> {
	let (input, type_name) = identifier(input)?;
	let (input, _) = space1(input)?;
	let (input, name) = identifier(input)?;
	let (input, _) = space0(input)?;
	let (input, _) = char(';')(input)?;
	Ok((
		input,
		Variable {
			active: true,
			kind: VariableKind::Regular,
			minified_name: None,
			name,
			type_name,
		},
	))
}

fn uniform_variable(input: &str) -> IResult<&str, Variable> {
	let (input, _) = tag("uniform")(input)?;
	let (input, _) = space1(input)?;
	let (input, type_name) = identifier(input)?;
	let (input, _) = space1(input)?;
	let (input, name) = identifier(input)?;
	let (input, _) = space0(input)?;
	let (input, _) = char(';')(input)?;
	Ok((
		input,
		Variable {
			active: true,
			kind: VariableKind::Uniform,
			minified_name: None,
			name,
			type_name,
		},
	))
}

pub fn variables(input: &str) -> IResult<&str, Vec<Variable>> {
	let (input, variables) = many0(take_unless(alt((
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
		map(const_variable, Some),
		map(regular_variable, Some),
		map(uniform_variable, Some),
	))))(input)?;
	Ok((input, variables.into_iter().map(|(_, var)| var).collect()))
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_contents() {
		let contents = contents(
			r#"#version 450
#define foo bar
prolog code
#pragma shiba common
common code
#pragma shiba vertex 42
vertex code
"#,
		);

		assert_eq!(
			contents,
			Ok((
				"vertex code\n",
				(
					Some("450"),
					vec![
						("#define foo bar\nprolog code\n", Section::Common),
						("common code\n", Section::Vertex(42)),
					]
				)
			))
		);
	}

	#[test]
	fn test_identifier() {
		assert_eq!(
			identifier("uniformVar0"),
			Ok(("", "uniformVar0".to_string()))
		);
	}

	#[test]
	fn test_variables() {
		let variables = variables(
			r#"precision mediump float;
float regularVar;
#define foo bar
const float constVar = 42.;
uniform float uniformVar0;
uniform float uniformVar1;
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
						minified_name: None,
						name: "regularVar".to_string(),
						type_name: "float".to_string(),
					},
					Variable {
						active: true,
						kind: VariableKind::Const("42.".to_string()),
						minified_name: None,
						name: "constVar".to_string(),
						type_name: "float".to_string(),
					},
					Variable {
						active: true,
						kind: VariableKind::Uniform,
						minified_name: None,
						name: "uniformVar0".to_string(),
						type_name: "float".to_string(),
					},
					Variable {
						active: true,
						kind: VariableKind::Uniform,
						minified_name: None,
						name: "uniformVar1".to_string(),
						type_name: "float".to_string(),
					},
					Variable {
						active: true,
						kind: VariableKind::Uniform,
						minified_name: None,
						name: "uniformVar2".to_string(),
						type_name: "vec2".to_string(),
					}
				]
			))
		);
	}
}
