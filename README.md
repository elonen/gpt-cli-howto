# gpt-cli-howto

Simple command line oracle that answers your questions using OpenAI GPT API.
You need to get an API key from https://openai.com/ and set it in the
configuration file.

It lets you ask questions and get answers from the command line, either
interactively or in one-shot mode.

Configuration file is INI format, and by default is located at `~/.howto.ini`.

## CLI options

```
Configurable CLI chat assistant, powered by OpenAI language models
You need to configure your OpenAI API key in ~/.howto.ini

Positional arguments:
  topic     Topic of the question (optional)
  question  Question to ask (required)

Usage:
  howto [topic] <question> [flags]

Examples:

    Configuration file example:

    [default]
    openai_token = sk-1234567890123456789012345678901234567890
    ; --- These are optional: ---
    chat = true                     ; If true, wait for a new question after each answer
    model = "gpt-4-turbo-preview"
    temperature = 0.1
    cost_per_token = 0.000025
    subject_msg = "Topic '{}'. Help me with the following task:"
    priming_msg = "This is a chat where experts give tested, modern answers to Debian Linux,\n\
      Sysops, Proxmox and networking questions. Answers are very compact, well-indented Markdown.\n\
      They value ultra short answer and don't waste reader's time with greetings and long-winded\n\
      explanations - but will warn you when the answer might be dangerous, and explain if it's\n\
      potentially hard to understand even for a pro."

Flags:
      --config string   config file (default is $HOME/.howto.ini)
  -d, --debug           Enable debug logging
  -h, --help            help for howto
  -v, --version         version for howto
```
