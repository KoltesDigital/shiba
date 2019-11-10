use super::types::*;
use crate::parsers::{glsl::*, *};
use nom::{branch::*, bytes::complete::*, combinator::*, multi::*, IResult};

fn section(input: &str) -> IResult<&str, Section> {
	directive(alt((
		value(Section::Attributes, tag("attributes")),
		value(Section::Common, tag("common")),
		map(fragment_directive, Section::Fragment),
		value(Section::Outputs, tag("outputs")),
		value(Section::UniformArrays, tag("uniform_arrays")),
		value(Section::Variables, tag("variables")),
		value(Section::Varyings, tag("varyings")),
		map(vertex_directive, Section::Vertex),
	)))(input)
}

fn sections(input: &str) -> IResult<&str, Vec<(&str, Section)>> {
	many0(take_unless(map(section, Some)))(input)
}

pub fn contents(input: &str) -> IResult<&str, Vec<(&str, Section)>> {
	sections(input)
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
				vec![
					(
						"#version 450\n#define foo bar\nprolog code\n",
						Section::Common
					),
					("common code\n", Section::Vertex(42)),
				]
			))
		);
	}
}
