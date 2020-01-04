#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Directive<'a> {
	Attributes,
	Common,
	Fragment(&'a str),
	Outputs,
	ShaderUniformArrays,
	ShaderVariables,
	Varyings,
	Vertex(&'a str),
}
