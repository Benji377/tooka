package shared

import (
	"os"

	"github.com/rs/zerolog"
	"github.com/rs/zerolog/log"
	"gopkg.in/natefinch/lumberjack.v2"
)

var Log zerolog.Logger

func InitLogger() {
	logsDir := GetLogsDir()

	// Create logs directory if it doesn't exist
	if _, err := os.Stat(logsDir); os.IsNotExist(err) {
		if err := os.MkdirAll(logsDir, 0755); err != nil {
			log.Fatal().Err(err).Msg("Failed to create logs directory")
		}
	}

	logFilePath := logsDir + "/tooka_log.json"

	logFile := &lumberjack.Logger{
		Filename:   logFilePath,
		MaxSize:    10, // megabytes
		MaxBackups: 7,
		MaxAge:     28,   //days
		Compress:   true, // disabled by default
	}

	log.Logger = zerolog.New(logFile).With().Timestamp().Logger()
	Log = log.Logger
}
