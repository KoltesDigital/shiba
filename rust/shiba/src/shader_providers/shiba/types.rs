#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Directive {
	Always,
	Attributes,
	Common,
	Development,
	Fragment(usize),
	Outputs,
	Prolog,
	Varyings,
	Vertex(usize),
}
