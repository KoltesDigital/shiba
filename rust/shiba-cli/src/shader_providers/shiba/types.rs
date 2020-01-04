#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Directive<'a> {
	Attributes,
	Common,
	Fragment(&'a str),
	Outputs,
	Prolog,
	Varyings,
	Vertex(&'a str),
}
