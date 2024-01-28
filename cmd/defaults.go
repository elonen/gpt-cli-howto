package cmd

import (
	"fmt"
	"strings"

	"github.com/MakeNowJust/heredoc"
)

const NAME = "howto"
const VERSION = "0.2.0"

var DEFAULT_PRIMING_MSG = heredoc.Doc(`
    This is a chat where experts give tested, modern answers to Debian Linux,
    Sysops, Proxmox and networking questions. Answers are very compact, well-indented Markdown.
    They value ultra short answer and don't waste reader's time with greetings and long-winded
    explanations - but will warn you when the answer might be dangerous, and explain if it's
    potentially hard to understand even for a pro.
`)

var DEFAULT_SUBJECT_MSG = "Topic '{}'. Help me with the following task:"
var SHORT_DEC = "Configurable CLI chat assistant, powered by OpenAI language models"

var LONG_DESC = fmt.Sprintf(heredoc.Doc(`
    %[1]s
    You need to configure your OpenAI API key in ~/.%[2]s.ini    
    
    Positional arguments:
      topic     Topic of the question (optional)
      question  Question to ask (required)`), SHORT_DEC, NAME)

var CONFIG_EXAMPLE = fmt.Sprintf(`
    Configuration file example:

    [default]
    openai_token = sk-1234567890123456789012345678901234567890
    ; --- These are optional: ---
    chat = true                     ; If true, wait for a new question after each answer
    model = "gpt-4-turbo-preview"
    temperature = 0.1
    subject_msg = "%[1]s"
    priming_msg = "%[2]s"`,
	strings.TrimSpace(DEFAULT_SUBJECT_MSG), strings.ReplaceAll(strings.TrimSpace(DEFAULT_PRIMING_MSG), "\n", "\\n\\\n      "))
