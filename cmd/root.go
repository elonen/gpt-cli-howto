package cmd

import (
	"bufio"
	"context"
	"errors"
	"fmt"
	"io"
	"os"
	"strings"
	"time"

	"github.com/briandowns/spinner"
	"github.com/charmbracelet/glamour"
	openai "github.com/sashabaranov/go-openai"
	"github.com/spf13/cobra"
)

func RunCmd(cmd *cobra.Command, args []string) {
	topic := ""
	question := args[0]
	if len(args) == 2 {
		topic = strings.ReplaceAll(args[0], "\n", " ")
		question = strings.ReplaceAll(DEFAULT_SUBJECT_MSG, "{}", topic) + "\n" + args[1]
	}

	// Test that API key is set
	config.Default.OpenaiToken = strings.TrimSpace(config.Default.OpenaiToken)
	if config.Default.OpenaiToken == "" {
		fmt.Fprintf(os.Stderr, "ERROR: openai_token is not set (or is empty) in config file\n")
		os.Exit(1)
	}

	// Initiate the chat dialog
	dlg := []openai.ChatCompletionMessage{
		{
			Role:    openai.ChatMessageRoleSystem,
			Content: config.Default.PrimingMsg,
		},
		{
			Role:    openai.ChatMessageRoleUser,
			Content: question,
		},
	}

	cconf := openai.DefaultConfig(config.Default.OpenaiToken)
	if config.Default.BaseURL != "" {
		cconf.BaseURL = config.Default.BaseURL
	}

	c := openai.NewClientWithConfig(cconf)
	ctx := context.Background()

	// Render the Markdown content with ANSI styling
	renderer, err := glamour.NewTermRenderer(
		glamour.WithStandardStyle("dark"),
		glamour.WithEmoji(),
		glamour.WithWordWrap(70),
	)
	if err != nil {
		panic(err)
	}

	for {
		answer := PerformQuery(ctx, dlg, c)

		dlg = append(dlg, openai.ChatCompletionMessage{
			Role:    openai.ChatMessageRoleAssistant,
			Content: answer,
		})

		// Render the answer (Markdown) to ANSI
		ansi_txt, err := renderer.Render(string(answer))
		if err != nil {
			panic(err)
		}
		fmt.Print(ansi_txt)

		// Ask a new question if chat mode is enabled
		if config.Default.Chat {
			fmt.Println("\033[1m" + "Type a continuation question, or press Enter to quit: " + "\033[0m")
			new_question, err := bufio.NewReader(os.Stdin).ReadString('\n')
			if err != nil {
				fmt.Fprintf(os.Stderr, "ERROR reading stdin: %v\n", err)
				os.Exit(1)
			}

			q := strings.TrimSpace(new_question)
			if q == "" || q == "q" || q == "quit" || q == "exit" {
				break
			} else {
				dlg = append(dlg, openai.ChatCompletionMessage{
					Role:    openai.ChatMessageRoleUser,
					Content: q,
				})
			}
		} else {
			break
		}
	}
}

// PerformQuery makes a single "chat" query to the OpenAI LLM API.
// It shows a spinner while waiting for the answer, and returns the answer.
func PerformQuery(ctx context.Context, dlg []openai.ChatCompletionMessage, c *openai.Client) string {

	// Show a spinner while waiting for the answer
	spinner := spinner.New(spinner.CharSets[14], 100*time.Millisecond)
	spinner.Start()
	defer spinner.Stop()
	spinner.Suffix = " Connecting..."

	// Initiate the API call
	req := openai.ChatCompletionRequest{
		Model:       config.Default.Model,
		Temperature: float32(config.Default.Temperature),
		MaxTokens:   1000,
		Messages:    dlg,
		Stream:      true,
	}
	stream, err := c.CreateChatCompletionStream(ctx, req)
	if err != nil {
		fmt.Fprintf(os.Stderr, "API call error: %v\n", err)
		os.Exit(1)
	}
	defer stream.Close()

	// Read the answer from the stream
	n_tokens := 0
	answer := ""
	for {
		response, err := stream.Recv()
		if errors.Is(err, io.EOF) {
			break
		}
		spinner.Suffix = fmt.Sprintf(" Working (%d tokens)...", n_tokens)
		if err != nil {
			fmt.Fprintf(os.Stderr, "\nStream error: %v\n", err)
			fmt.Println("Answer so far:", answer)
			break
		}
		n_tokens += 1
		answer += response.Choices[0].Delta.Content
	}

	return answer
}

var cfgFile string

func init() {
	cobra.OnInitialize(initConfig)
	rootCmd.PersistentFlags().StringVar(&cfgFile, "config", "", "config file (default is $HOME/.howto.ini)")
	rootCmd.PersistentFlags().BoolP("debug", "d", false, "Enable debug logging")
}

var rootCmd = &cobra.Command{
	Use:     "howto [topic] <question>",
	Short:   SHORT_DEC,
	Long:    LONG_DESC,
	Example: CONFIG_EXAMPLE,
	Version: VERSION,
	Args:    cobra.RangeArgs(1, 2),
	Run:     RunCmd,
}

func Execute() {
	if err := rootCmd.Execute(); err != nil {
		fmt.Println(err)
		os.Exit(1)
	}
}
