use std::{path::{PathBuf, Path}, sync::{atomic::AtomicBool, Arc}};
use termimad;

use docopt::Docopt;
use anyhow::anyhow;
use log::{debug};
use serde::Deserialize;

const NAME: &'static str = env!("CARGO_BIN_NAME");  
const VERSION: &'static str = env!("CARGO_PKG_VERSION");

const USAGE: &'static str = r#"
Command-line chat assistant, powered by OpenAI language models.
You need to configure your OpenAI API key in ~/.{NAME}.ini

The example configuration file below primes the model to
be a Linux sysops assistant, but you can change it to anything.

Usage:
  {NAME} [options] <question>
  {NAME} [options] <subject> <question>
  {NAME} (-h | --help)
  {NAME} (-v | --version)

Required:

Options:
 -c --config <inifile>  Config file [default: ~/.{NAME}.ini]
 -d --debug             Enable debug logging
 -h --help              Show this screen
 -v --version           Show version ("{VERSION}")


Example configuration file:

  [default]
  priming_msg = "This is a chat where an expert answers Debian Linux, Sysops and networking questions. Answers are very compact and well-structured in Markdown, not overly friendly. CLI one-liners are preferred, when possible. Warnings given when the suggested commands might be dangerous."
  subject_msg = "Topic '{}'. Help me with the following task:"
  model = "gpt-3.5-turbo"
  openai_token = sk-1234567890123456789012345678901234567890
  cost_per_token = 0.000002
"#;


#[derive(Debug, Deserialize)]
struct Args {
  arg_subject: Option<String>,
  arg_question: String,
  flag_config: PathBuf,
  flag_debug: bool,
  flag_version: bool,
}

struct Config {
  priming_msg: String,
  subject_msg: String,
  model: String,
  openai_token: String,
  cost_per_token: f64,
}

fn read_config_file(config_file: &Path) -> anyhow::Result<Config> {
    debug!("Reading config file: {:?}", &config_file);
    let config_file = shellexpand::tilde(config_file.to_str().unwrap()).into_owned(); // Replace ~ with current user's home directory

    let config = ini::Ini::load_from_file(&config_file)
      .map_err(|e| anyhow!("Error reading config file {:?}: {}", &config_file, e))?;

    let def = config.section(Some("default".to_owned())).ok_or_else(|| anyhow!("Missing default section in config file"))?;
    Ok(Config {
        priming_msg: def.get("priming_msg").ok_or_else(|| anyhow!("Missing priming_msg in config file"))?.to_string(),
        subject_msg: def.get("subject_msg").ok_or_else(|| anyhow!("Missing subject_msg in config file"))?.to_string(),
        model: def.get("model").ok_or_else(|| anyhow!("Missing model in config file"))?.to_string(),
        openai_token: def.get("openai_token").ok_or_else(|| anyhow!("Missing openai_token in config file"))?.to_string(),
        cost_per_token: def.get("cost_per_token").ok_or_else(|| anyhow!("Missing cost_per_token in config file"))?.parse()?,
    })
}

/// Show a progress bar while the request is being performed.
/// Terminate when terminate_signal is given, and clean up the progress bar.
fn progress_bar_thread(terminate_signal: Arc<AtomicBool>) -> anyhow::Result<()> {
  let pb = indicatif::ProgressBar::new_spinner();
  let style = indicatif::ProgressStyle::default_spinner()
    .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏ ")
    .template("{spinner:.green} {msg}")?;
  pb.set_style(style);
  pb.set_message("Working...");

  while !terminate_signal.load(std::sync::atomic::Ordering::Relaxed) {
    std::thread::sleep(std::time::Duration::from_millis(100));
    pb.tick();
  }
  pb.finish_and_clear();
  Ok(())
}


fn perform_request(conf: &Config, question: String, debug: bool) -> anyhow::Result<(String, Option<u64>)> {
  let url = "https://api.openai.com/v1/chat/completions";
  let authorization = format!("Bearer {}", conf.openai_token);

  let req_body_json = serde_json::json!({
    "model": conf.model,
    "messages": [
      {"role": "system", "content": conf.priming_msg},
      {"role": "user", "content": question},
    ]
  });

  let res = reqwest::blocking::Client::builder()
    .timeout(std::time::Duration::from_secs(120))
    .build()?
    .post(url)
    .header("Content-Type", "application/json")
    .header("Authorization", authorization)
    .json(&req_body_json)
    .send()?; 

  if debug { eprintln!("RESPONSE: {:?}", res); }

  let res_body = res.text()?;
  let res_body_json: serde_json::Value = serde_json::from_str(&res_body)?;

  let tokens = res_body_json["usage"]["total_tokens"].as_u64();

  // print all "choices" messages
  let mut answ = String::new();
  for choice in res_body_json["choices"].as_array().unwrap() {
    let message = choice["message"]["content"].as_str().unwrap();
    answ.push_str(message);
  }
  Ok((answ, tokens))
}

fn main() -> anyhow::Result<()>
{
  let args: Args = Docopt::new(USAGE.replace("{NAME}", NAME).replace("{VERSION}", VERSION))
  .and_then(|d| d.deserialize())
  .unwrap_or_else(|e| e.exit());

  if args.flag_version {
    println!("{}", VERSION);
    return Ok(());
  }

  env_logger::builder()
      .filter_level(if args.flag_debug {log::LevelFilter::Debug} else {log::LevelFilter::Info})
      .init();

  let conf = read_config_file(&args.flag_config)?;

  // Format the question
  let mut question = args.arg_question.to_string();
  if let Some(subject) = args.arg_subject {
    let subject = conf.subject_msg.replace("{}", &subject);
    question = format!("{} {}", subject, question);
  }

  // Start a progress bar thread
  let terminate_signal = Arc::new(AtomicBool::new(false));
  let ts = terminate_signal.clone();
  let pb_thread = std::thread::spawn(move || {
    progress_bar_thread(ts).unwrap();
  });

  let res= perform_request(&conf, question, args.flag_debug);

  // Stop the progress bar thread
  terminate_signal.store(true, std::sync::atomic::Ordering::Relaxed);
  let _ = pb_thread.join();

  match res {
    Ok((answer_md, tokens)) => {
      termimad::print_text(&answer_md);
      if let Some(tokens) = tokens {
        let cost = tokens as f64 * conf.cost_per_token;
        println!("\x1b[38;5;208mCost: {} tokens, {:.4} USD\x1b[0m", tokens, cost);
      }
    },
    Err(e) => {
      eprintln!("Error: {}", e);
    }
  }
  
  Ok(())
}
