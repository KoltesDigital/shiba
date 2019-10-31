#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Section {
	Attributes,
	Common,
	Fragment(usize),
	Outputs,
	Prolog,
	Varyings,
	Vertex(usize),
}
