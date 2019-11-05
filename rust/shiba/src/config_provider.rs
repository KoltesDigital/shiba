use crate::paths;
use std::fs;
use std::path::PathBuf;

fn merge(a: &mut serde_yaml::Value, b: &serde_yaml::Value) {
	match (a, b) {
		(&mut serde_yaml::Value::Mapping(ref mut a), &serde_yaml::Value::Mapping(ref b)) => {
			for (k, v) in b {
				match a.get_mut(k) {
					Some(o) => {
						merge(o, v);
					}
					None => {
						a.insert(k.clone(), v.clone());
					}
				}
			}
		}
		(a, b) => {
			*a = b.clone();
		}
	}
}

#[derive(Debug)]
pub struct ConfigProvider {
	config: serde_yaml::Value,
}

impl ConfigProvider {
	pub fn load() -> Result<Self, String> {
		let mut config = serde_yaml::Value::Mapping(serde_yaml::Mapping::default());

		let paths = vec![
			(*paths::USER_SETTINGS).join("config.yml"),
			PathBuf::from("config.yml"),
		];

		for path in paths {
			if path.as_path().exists() {
				println!("Loading config from {}.", path.to_str().unwrap());
				let contents = fs::read_to_string(path)
					.map_err(|_| "Failed to open config file.".to_string())?;
				let value: serde_yaml::Value = serde_yaml::from_str(&contents)
					.map_err(|err| format!("Failed to parse: {}.", err))?;
				merge(&mut config, &value);
			}
		}

		let config_provider = ConfigProvider { config };

		Ok(config_provider)
	}

	pub fn get<O: serde::de::DeserializeOwned>(&self) -> Result<O, String> {
		let mut value = serde_yaml::Value::Mapping(serde_yaml::Mapping::default());
		merge(&mut value, &self.config);
		serde_yaml::from_value(value).map_err(|err| err.to_string())
	}

	pub fn get_default<I: serde::Serialize, O: serde::de::DeserializeOwned>(
		&self,
		defaults: I,
	) -> Result<O, String> {
		let mut value =
			serde_yaml::to_value(&defaults).map_err(|_| "Cannot load defaults.".to_string())?;
		merge(&mut value, &self.config);
		serde_yaml::from_value(value).map_err(|_| "Cannot generate.".to_string())
	}
}
