use config::ini_format_multiline_str;
use docopt::Docopt;
use serde::Deserialize;
use std::{path::PathBuf, sync::mpsc};
use termimad;

use crate::query::perform_streaming_request;

mod config;
mod query;

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
  openai_token = sk-1234567890123456789012345678901234567890
  ; --- These are optional: ---
  model = "gpt-3.5-turbo"
  temperature = 0.4
  cost_per_token = 0.000002       ; No default, won't show query cost if missing
  subject_msg = "{SUBJECT_MSG}"
  priming_msg = "{PRIMING_MSG}"
"#;

#[derive(Debug, Deserialize)]
struct Args {
    arg_subject: Option<String>,
    arg_question: String,
    flag_config: PathBuf,
    flag_debug: bool,
    flag_version: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {

    let args: Args = Docopt::new(USAGE
            .replace("{NAME}", NAME)
            .replace("{VERSION}", VERSION)
            .replace("{PRIMING_MSG}", &ini_format_multiline_str(config::DEFAULT_PRIMING_MSG))
            .replace("{SUBJECT_MSG}", &ini_format_multiline_str(config::DEFAULT_SUBJECT_MSG)))
        .and_then(|d| d.deserialize())
        .unwrap_or_else(|e| e.exit());

    if args.flag_version {
        println!("{}", VERSION);
        return Ok(());
    }

    env_logger::builder()
        .filter_level(if args.flag_debug {
            log::LevelFilter::Debug
        } else {
            log::LevelFilter::Info
        })
        .init();

    let conf = config::read_config_file(&args.flag_config)?;

    // Format the question
    let mut question = args.arg_question.to_string();
    if let Some(subject) = args.arg_subject {
        let subject = conf.subject_msg.replace("{}", &subject);
        question = format!("{} {}", subject, question);
    }

    let (evt_tx, evt_rx) = mpsc::channel::<String>();
    let pb_thread = tokio::spawn(async move { progress_bar_thread(evt_rx) });
    let req = tokio::spawn(perform_streaming_request(conf.clone(), question, evt_tx));

    pb_thread.await??; // Wait for the progress bar to cleanup itself

    match req.await? {
        Ok((answer_md, tokens)) => {
            println!("\n");
            termimad::print_text(&answer_md.trim());
            if let Some(tokens) = tokens {
                if let Some(cost_per_token) = conf.cost_per_token {
                    let cost = tokens as f64 * cost_per_token;
                    print_orange(format!("\n(Cost: {} tokens, {:.4} USD)", tokens, cost));
                }
            }
        }
        Err(e) => {
            eprintln!("Error: {}", e);
        }
    }

    Ok(())
}

fn print_orange(s: String) {
    let mut md = termimad::MadSkin::default();
    md.set_fg(termimad::ansi(3));
    md.print_text(&s);
  }

/// Show a progress bar while the request is being performed.
/// Terminate when terminate_signal is given, and clean up the progress bar.
fn progress_bar_thread(recv: mpsc::Receiver<String>) -> anyhow::Result<()> {
    let pb = indicatif::ProgressBar::new_spinner();
    let style = indicatif::ProgressStyle::default_spinner()
        .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏ ")
        .template("{spinner:.green} {msg}")?;
    pb.set_style(style);
    pb.set_message("Connecting...");

    let mut total_tokens = 0;
    while let Ok(_msg) = recv.recv() {
        total_tokens += 1;
        pb.set_message(format!("Working ({} tokens)...", total_tokens));
        std::thread::sleep(std::time::Duration::from_millis(100));
        pb.tick();
    }

    pb.finish_and_clear();
    Ok(())
}
