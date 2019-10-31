use crate::types::{ProjectDescriptor, ShaderDescriptor};

pub trait ShaderProvider {
	fn provide_shader(
		&self,
		project_descriptor: &ProjectDescriptor,
	) -> Result<ShaderDescriptor, String>;
}
