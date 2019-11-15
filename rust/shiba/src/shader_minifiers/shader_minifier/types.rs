#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Directive {
	Attributes,
	Common,
	Fragment(usize),
	Outputs,
	UniformArrays,
	Variables,
	Varyings,
	Vertex(usize),
}
