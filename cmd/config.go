package cmd

import (
	"fmt"
	"log"
	"os"

	"github.com/mitchellh/mapstructure"
	"github.com/spf13/cobra"
	"github.com/spf13/viper"
)

type Config struct {
	Default struct {
		OpenaiToken string  `mapstructure:"openai_token"`
		BaseURL     string  `mapstructure:"base_url"`
		Chat        bool    `mapstructure:"chat"`
		Model       string  `mapstructure:"model"`
		Temperature float32 `mapstructure:"temperature"`
		SubjectMsg  string  `mapstructure:"subject_msg"`
		PrimingMsg  string  `mapstructure:"priming_msg"`
	}
}

var config Config

func initConfig() {
	if cfgFile != "" {
		viper.SetConfigFile(cfgFile) // Use config file from cli option.
	} else {
		home, err := os.UserHomeDir() // Read default config
		cobra.CheckErr(err)
		viper.AddConfigPath(home)
		viper.SetConfigName(".howto")
	}
	viper.SetConfigType("ini")
	viper.SetTypeByDefaultValue(true)
	viper.SetDefault("default.openai_token", "")
	viper.SetDefault("default.base_url", "https://api.openai.com/v1")
	viper.SetDefault("default.chat", true)
	viper.SetDefault("default.model", "gpt-4-turbo-preview")
	viper.SetDefault("default.temperature", 0.1)
	viper.SetDefault("default.subject_msg", DEFAULT_SUBJECT_MSG)
	viper.SetDefault("default.priming_msg", DEFAULT_PRIMING_MSG)

	// If a config file is found, read it in
	if err := viper.ReadInConfig(); err != nil {
		fmt.Fprintf(os.Stderr, "ERROR reading config file: %s\n", err)
		os.Exit(1)
	}
	// Apply defaults and unmarshal config
	settings := viper.AllSettings()
	if err := mapstructure.Decode(settings, &config); err != nil {
		log.Fatalf("Error decoding config: %s", err)
	}
}
