use super::types::*;
use crate::parsers::{glsl::*, *};
use nom::{
	branch::*, bytes::complete::*, character::complete::*, combinator::*, multi::*, sequence::*,
	IResult,
};

fn section(input: &str) -> IResult<&str, Directive> {
	directive(alt((
		value(Directive::Attributes, tag("attributes")),
		value(Directive::Common, tag("common")),
		map(fragment_directive, Directive::Fragment),
		value(Directive::Outputs, tag("outputs")),
		value(Directive::Varyings, tag("varyings")),
		map(vertex_directive, Directive::Vertex),
	)))(input)
}

fn sections(input: &str) -> IResult<&str, Vec<(&str, Directive)>> {
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

pub type Contents<'a> = (Option<&'a str>, Vec<(&'a str, Directive<'a>)>);

pub fn contents(input: &str) -> IResult<&str, Contents> {
	tuple((opt(version), sections))(input)
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
#pragma shiba vertex id
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
						("#define foo bar\nprolog code\n", Directive::Common),
						("common code\n", Directive::Vertex("id")),
					]
				)
			))
		);
	}
}
