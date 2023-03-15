use anyhow::anyhow;
use log::debug;
use std::path::Path;

pub const DEFAULT_PRIMING_MSG: &'static str = r#"
This is a chat where experts give tested, modern answers to Debian Linux,
Sysops, Proxmox and networking questions. Answers are very compact, well-indented Markdown.
They value ultra short answer and don't waste reader's time with greetings and long-winded
explanations - but will warn you when the answer might be dangerous, and explain if it's
potentially hard to understand even for a pro.
"#;

pub const DEFAULT_SUBJECT_MSG: &'static str = "Topic '{}'. Help me with the following task:";

#[derive(Clone)]
pub struct Config {
    pub openai_token: String,
    pub priming_msg: String, // default: see USAGE
    pub subject_msg: String, // default: see USAGE
    pub model: String,       // default: "gpt-3.5-turbo"
    pub cost_per_token: Option<f64>,
    pub temperature: f64, // default 0.4
    pub chat: bool,       // default: true
}

pub fn read_config_file(config_file: &Path) -> anyhow::Result<Config> {
    debug!("Reading config file: {:?}", &config_file);
    let config_file = shellexpand::tilde(config_file.to_str().unwrap()).into_owned(); // Replace ~ with current user's home directory

    let config = ini::Ini::load_from_file(&config_file)
        .map_err(|e| anyhow!("Error reading config file {:?}: {}", &config_file, e))?;

    let def = config
        .section(Some("default".to_owned()))
        .ok_or_else(|| anyhow!("Missing default section in config file"))?;
    Ok(Config {
        openai_token: def
            .get("openai_token")
            .ok_or_else(|| anyhow!("Missing openai_token in config file"))?
            .to_string(),
        priming_msg: def
            .get("priming_msg")
            .unwrap_or(DEFAULT_PRIMING_MSG)
            // remove leading and trailing whitespaces per line
            .lines()
            .map(|s| s.trim())
            .collect::<Vec<&str>>()
            .join("\n"),
        subject_msg: def
            .get("subject_msg")
            .unwrap_or(DEFAULT_SUBJECT_MSG)
            .lines()
            .map(|s| s.trim())
            .collect::<Vec<&str>>()
            .join("\n"),
        model: def.get("model").unwrap_or("gpt-3.5-turbo").to_string(),
        cost_per_token: def.get("cost_per_token").map(|s| s.parse()).transpose()?,
        temperature: def.get("temperature").unwrap_or("0.4").parse()?,
        chat: def.get("chat").unwrap_or("true").parse()?,
    })
}

pub fn ini_format_multiline_str(s: &str) -> String {
    s.trim()
        .lines()
        .map(|s| format!("    {}", s))
        .collect::<Vec<String>>()
        .join("\\n\\\n")
        .trim_start()
        .to_string()
}
