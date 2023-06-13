use config::ini_format_multiline_str;
use docopt::Docopt;
use indicatif::{ProgressStyle, ProgressBar};
use query::ChatRole;
use serde::Deserialize;
use std::{path::PathBuf, sync::mpsc, fmt::Debug};
use termimad;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
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
  chat = true                     ; If true, wait for a new question after each answer
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
    let args: Args = Docopt::new(
        USAGE
            .replace("{NAME}", NAME)
            .replace("{VERSION}", VERSION)
            .replace(
                "{PRIMING_MSG}",
                &ini_format_multiline_str(config::DEFAULT_PRIMING_MSG),
            )
            .replace(
                "{SUBJECT_MSG}",
                &ini_format_multiline_str(config::DEFAULT_SUBJECT_MSG),
            ),
    )
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

    // Prime the assistant
    let mut history: Vec<(ChatRole, String)> = vec![(ChatRole::System, conf.priming_msg.clone())];

    // Format first question
    let mut question = args.arg_question.to_string();
    if let Some(subject) = args.arg_subject {
        let subject = conf.subject_msg.replace("{}", &subject);
        question = format!("{} {}", subject, question);
    }

    loop {
        history.push((ChatRole::User, question.clone()));

        let (evt_tx, evt_rx) = mpsc::channel::<String>();
        let pb_thread = tokio::spawn(async move { progress_bar_thread(evt_rx) });
        let req = tokio::spawn(perform_streaming_request(
            conf.clone(),
            history.clone(),
            evt_tx,
        ));

        pb_thread.await??; // Wait for the progress bar to cleanup itself

        match req.await? {
            Ok((answer_md, tokens)) => {
                history.push((ChatRole::Assistant, answer_md.clone()));

                let skin = termimad::MadSkin::default();
                let (tw, _) = termimad::terminal_size();
                let tw = std::cmp::min(tw, 80)-2;
                let fmt_txt = termimad::FmtText::from(&skin, &answer_md, Some(tw as usize));
                let indented = fmt_txt.to_string().lines()
                    .map(|l| format!("  {}", l)).collect::<Vec<_>>().join("\n");
                println!("\n{}", &indented);

                if let Some(tokens) = tokens {
                    if let Some(cost_per_token) = conf.cost_per_token {
                        let cost = tokens as f64 * cost_per_token;
                        print_orange(format!("\n(Cost: {} tokens, {:.4} USD)", tokens, cost));
                    }
                }

                if !conf.chat {
                    break;
                }

                question = match prompt_for_continuation()? {
                    Some(q) => q,
                    None => break,
                }
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                break;
            }
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
    let pb = ProgressBar::new_spinner();
    let style = ProgressStyle::default_spinner()
        .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏ ")
        .template("{spinner:.green} {msg}")?;
    pb.set_style(style);
    pb.set_message("Connecting...");

    let mut toks = 0;
    while let Ok(_msg) = recv.recv() {
        toks += 1;
        pb.set_message(format!("Working ({} tokens)...", toks));
        pb.tick();
    }

    pb.finish_and_clear();
    Ok(())
}

// Use termimad to ask the user if they want to continue
fn prompt_for_continuation() -> anyhow::Result<Option<String>> {
    let mut md = termimad::MadSkin::default();
    md.set_fg(termimad::ansi(6));
    let q = "Type a continuation question, or press Enter to quit";
    md.print_text(q);

    match DefaultEditor::new()?.readline("") {
        Ok(input) => {
            let input = input.trim();
            if input.is_empty() || input == "q" || input == "quit" {
                Ok(None)
            } else {
                Ok(Some(input.to_string()))
            }
        }
        Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => Ok(None),
        Err(err) => Err(anyhow::anyhow!(format!("Error: {:?}", err))),
    }
}
