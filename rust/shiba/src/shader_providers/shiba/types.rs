#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Directive {
	Attributes,
	Common,
	Fragment(usize),
	Outputs,
	Prolog,
	Varyings,
	Vertex(usize),
}
