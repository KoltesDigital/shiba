use super::*;
use crate::types::{
	ConstVariable, UniformAnnotationControlDescriptor, UniformAnnotationKind, UniformVariable,
	Variable, VariableKind,
};
use nom::{
	branch::*, bytes::complete::*, character::complete::*, combinator::*, multi::*, sequence::*,
	IResult,
};
use std::collections::BTreeMap;
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
				let slice = slice::from_raw_parts(ptr, 1 + rest.len());
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
					kind: VariableKind::Const(ConstVariable {
						value: value.to_string(),
					}),
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

pub fn uniform_control_annotation_parameters<'a>(
	input: &'a str,
) -> IResult<&'a str, BTreeMap<String, String>> {
	map(
		separated_list(
			char(','),
			map(
				tuple((
					space0,
					identifier,
					space0,
					char('='),
					space0,
					alt((
						map(
							tuple((
								take_while_m_n(1, 1, |c: char| c == '('),
								take_while(|c: char| c != ')'),
								char(')'),
							)),
							|(left, middle, _): (_, &str, _)| {
								let ptr = left.as_ptr();
								unsafe {
									let slice = slice::from_raw_parts(ptr, 1 + middle.len() + 1);
									str::from_utf8(slice)
								}
								.unwrap()
							},
						),
						map(
							tuple((
								take_while_m_n(1, 1, |c: char| c == '"'),
								take_while(|c: char| c != '"'),
								char('"'),
							)),
							|(_, middle, _): (_, &str, _)| middle,
						),
						take_while(|c: char| c != ',' && c != ')'),
					)),
					space0,
				)),
				|(_, key, _, _, _, value, _)| (key.to_string(), value.to_string()),
			),
		),
		|tuples| tuples.into_iter().collect(),
	)(input)
}

pub fn uniform_annotation<'a>(input: &'a str) -> IResult<&'a str, UniformAnnotationKind> {
	alt((
		map(
			tuple((
				tag("control"),
				space0,
				opt(delimited(
					char('('),
					uniform_control_annotation_parameters,
					char(')'),
				)),
			)),
			|(_, _, parameters)| {
				UniformAnnotationKind::Control(UniformAnnotationControlDescriptor {
					parameters: parameters.unwrap_or_default(),
				})
			},
		),
		value(
			UniformAnnotationKind::InverseProjection,
			tag("inverse-projection"),
		),
		value(UniformAnnotationKind::InverseView, tag("inverse-view")),
		value(UniformAnnotationKind::Projection, tag("projection")),
		value(
			UniformAnnotationKind::ResolutionHeight,
			tag("resolution-height"),
		),
		value(
			UniformAnnotationKind::ResolutionWidth,
			tag("resolution-width"),
		),
		value(UniformAnnotationKind::Time, tag("time")),
		value(UniformAnnotationKind::View, tag("view")),
	))(input)
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
			opt(map(
				tuple((
					space0,
					tag("//"),
					space0,
					tag("shiba"),
					space1,
					separated_list(tuple((char(','), space0)), uniform_annotation),
				)),
				|(_, _, _, _, _, annotations)| annotations,
			)),
		)),
		|(_, _, type_name, _, list, _, annotations)| {
			list.into_iter()
				.map(|(name, length)| Variable {
					active: true,
					kind: VariableKind::Uniform(UniformVariable {
						annotations: annotations.clone().unwrap_or_default(),
					}),
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
	use crate::types::UniformAnnotationControlDescriptor;

	#[test]
	fn test_identifier() {
		assert_eq!(identifier("uniformVar0"), Ok(("", "uniformVar0")));
	}

	#[test]
	fn test_uniform_control_annotation_parameters() {
		let parameters = uniform_control_annotation_parameters(
			"default=(.5,.5,.5), min=0, max=1, subtype=color)",
		);

		let mut expected_parameters = BTreeMap::new();
		expected_parameters.insert("default".to_string(), "(.5,.5,.5)".to_string());
		expected_parameters.insert("min".to_string(), "0".to_string());
		expected_parameters.insert("max".to_string(), "1".to_string());
		expected_parameters.insert("subtype".to_string(), "color".to_string());

		assert_eq!(parameters, Ok((")", expected_parameters)));
	}

	#[test]
	fn test_uniform_control_annotation_parameters_with_spaces() {
		let parameters = uniform_control_annotation_parameters(
			" default = (0, 1) , description = \"foo, bar\" , min = 0 , max = 1 )",
		);

		let mut expected_parameters = BTreeMap::new();
		expected_parameters.insert("default".to_string(), "(0, 1)".to_string());
		expected_parameters.insert("description".to_string(), "foo, bar".to_string());
		expected_parameters.insert("min".to_string(), "0 ".to_string());
		expected_parameters.insert("max".to_string(), "1 ".to_string());

		assert_eq!(parameters, Ok((")", expected_parameters)));
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
uniform float uniformVar1[4]; // simple comment
uniform vec3 uniformVar2; // shiba control(default=(.5,.5,.5), min=0, max=1, subtype=color)
uniform bool uniformVar3;
"#,
		);

		let mut expected_control_annotation_parameters = BTreeMap::new();
		expected_control_annotation_parameters
			.insert("default".to_string(), "(.5,.5,.5)".to_string());
		expected_control_annotation_parameters.insert("min".to_string(), "0".to_string());
		expected_control_annotation_parameters.insert("max".to_string(), "1".to_string());
		expected_control_annotation_parameters.insert("subtype".to_string(), "color".to_string());

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
						kind: VariableKind::Const(ConstVariable {
							value: "42.".to_string()
						}),
						length: None,
						minified_name: None,
						name: "constVar0".to_string(),
						type_name: "float".to_string(),
					},
					Variable {
						active: true,
						kind: VariableKind::Const(ConstVariable {
							value: "1337.".to_string()
						}),
						length: None,
						minified_name: None,
						name: "constVar1".to_string(),
						type_name: "float".to_string(),
					},
					Variable {
						active: true,
						kind: VariableKind::Uniform(UniformVariable {
							annotations: vec![]
						}),
						length: None,
						minified_name: None,
						name: "uniformVar0".to_string(),
						type_name: "float".to_string(),
					},
					Variable {
						active: true,
						kind: VariableKind::Uniform(UniformVariable {
							annotations: vec![]
						}),
						length: Some(4),
						minified_name: None,
						name: "uniformVar1".to_string(),
						type_name: "float".to_string(),
					},
					Variable {
						active: true,
						kind: VariableKind::Uniform(UniformVariable {
							annotations: vec![UniformAnnotationKind::Control(
								UniformAnnotationControlDescriptor {
									parameters: expected_control_annotation_parameters,
								}
							)]
						}),
						length: None,
						minified_name: None,
						name: "uniformVar2".to_string(),
						type_name: "vec3".to_string(),
					},
					Variable {
						active: true,
						kind: VariableKind::Uniform(UniformVariable {
							annotations: vec![]
						}),
						length: None,
						minified_name: None,
						name: "uniformVar3".to_string(),
						type_name: "bool".to_string(),
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
