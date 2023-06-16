use serde_derive::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Settings {
    pub api_key: String,
    pub api_url: String,
    pub role_list: Vec<Role>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Role {
    pub name: String,
    pub prompt: String,
    pub icon_base64: String,
}

/// `MyConfig` implements `Default`
impl ::std::default::Default for Settings {
    fn default() -> Self {
        Self {
            api_key: "".into(),
            api_url: "https://api.openai.com/v1/chat/completions".into(),
            role_list: Vec::from_iter([
                Role {
                    name: "XXXGPT".into(),
                    prompt: "You are XXXGPT, an ai model".into(),
                    icon_base64: "".into(),
                },
                Role {
                    name: "ChatGPT".into(),
                    prompt: "You are ChatGPT, an ai model".into(),
                    icon_base64: "".into(),
                },
                Role {
                    name: "Translator".into(),
                    prompt: "You are TranGPT dedicated for translating between Chinese and English"
                        .into(),
                    icon_base64: "".into(),
                },
                Role {
                    name: "Last".into(),
                    prompt: "You are LastGPT dedicated for translating between Chinese and English"
                        .into(),
                    icon_base64: "".into(),
                },
            ]),
        }
    }
}
