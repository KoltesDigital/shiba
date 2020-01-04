use super::types::*;
use crate::parsers::{glsl::*, *};
use nom::{branch::*, bytes::complete::*, combinator::*, multi::*, IResult};

fn section(input: &str) -> IResult<&str, Directive> {
	directive(alt((
		value(Directive::Attributes, tag("attributes")),
		value(Directive::Common, tag("common")),
		map(fragment_directive, Directive::Fragment),
		value(Directive::Outputs, tag("outputs")),
		value(Directive::ShaderUniformArrays, tag("uniform_arrays")),
		value(Directive::ShaderVariables, tag("variables")),
		value(Directive::Varyings, tag("varyings")),
		map(vertex_directive, Directive::Vertex),
	)))(input)
}

fn sections(input: &str) -> IResult<&str, Vec<(&str, Directive)>> {
	many0(take_unless(map(section, Some)))(input)
}

pub fn contents(input: &str) -> IResult<&str, Vec<(&str, Directive)>> {
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
#pragma shiba vertex id
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
						Directive::Common
					),
					("common code\n", Directive::Vertex("id")),
				]
			))
		);
	}
}
