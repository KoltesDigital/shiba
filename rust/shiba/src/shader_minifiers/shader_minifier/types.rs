#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Section {
	Attributes,
	Common,
	Fragment(usize),
	Outputs,
	UniformArrays,
	Variables,
	Varyings,
	Vertex(usize),
}
