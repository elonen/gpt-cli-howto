# gpt-cli-howto

Simple command line oracle that answers your questions using OpenAI GPT API.
You need to get an API key from https://openai.com/ and set it in the
configuration file.

It lets you ask questions and get answers from the command line, either
interactively or in one-shot mode.

Configuration file is INI format, and by default is located at `~/.howto.ini`.

## CLI options

```
Command-line chat assistant, powered by OpenAI language models.
You need to configure your OpenAI API key in ~/.howto.ini

The example configuration file below primes the model to
be a Linux sysops assistant, but you can change it to anything.

Usage:
  howto [options] <question>
  howto [options] <subject> <question>
  howto (-h | --help)
  howto (-v | --version)

Required:

Options:
 -c --config <inifile>  Config file [default: ~/.howto.ini]
 -d --debug             Enable debug logging
 -h --help              Show this screen
 -v --version           Show version ("0.1.0")


Example configuration file:

  [default]
  openai_token = sk-1234567890123456789012345678901234567890
  ; --- These are optional: ---
  chat = true                     ; If true, wait for a new question after each answer
  model = "gpt-3.5-turbo"
  temperature = 0.4
  cost_per_token = 0.000002       ; No default, won't show query cost if missing
  subject_msg = "Topic '{}'. Help me with the following task:"
  priming_msg = "This is a chat where an expert answer Debian Linux, Sysops and\n\
    networking questions. Answers are very compact, well-indented Markdown.\n\
    To avoid wasting reader's time, greetings and other extra text is avoided.\n\
    One-liner answers are totally fine, when no explanation is needed,\n\
    but warnings are given when the suggested commands might be dangerous."
```

