use config::{Config, ConfigError, File, FileFormat};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Clone)]
pub struct Log {
    pub level: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Server {
    pub port: u16,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Character {
    pub path: String,
    #[serde(default)]
    pub content: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Tee {
    pub tee_host: String,
    pub basic_token: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Icp {
    pub api_host: String,
    pub object_store_canister: String,
}

/// Configuration for the LLM should be encrypted and stored in the ICP COSE canister.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Llm {
    #[serde(default)]
    pub deepseek_api_key: String,
    #[serde(default)]
    pub cohere_api_key: String,
    #[serde(default)]
    pub cohere_embedding_model: String,
    #[serde(default)]
    pub openai_api_key: String,
    #[serde(default)]
    pub openai_embedding_model: String,
    #[serde(default)]
    pub openai_completion_model: String,
}

/// Configuration for the X should be encrypted and stored in the ICP COSE canister.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct X {
    pub username: String,
    pub password: String,
    pub email: Option<String>,
    pub two_factor_auth: Option<String>,
    pub cookie_string: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Conf {
    pub character: Character,
    pub log: Log,
    pub server: Server,
    pub llm: Llm,
    pub tee: Tee,
    pub icp: Icp,
    pub x: X,
}

impl Conf {
    pub fn new() -> Result<Self, ConfigError> {
        let file_name =
            std::env::var("CONFIG_FILE_PATH").unwrap_or_else(|_| "./config.toml".into());
        let mut cfg = Self::from(&file_name)?;
        cfg.character.content = std::fs::read_to_string(&cfg.character.path)
            .map_err(|err| ConfigError::NotFound(err.to_string()))?;
        Ok(cfg)
    }

    pub fn from(file_name: &str) -> Result<Self, ConfigError> {
        let builder = Config::builder().add_source(File::new(file_name, FileFormat::Toml));
        builder.build()?.try_deserialize::<Conf>()
    }
}
