package core

import (
	"io"
	"os"

	"github.com/rs/zerolog"
	"github.com/rs/zerolog/log"
	"gopkg.in/natefinch/lumberjack.v2"
)

var Log zerolog.Logger

func InitLogger() {
	logFile := &lumberjack.Logger{
		Filename:   LogsDir + "/tooka.log",
		MaxSize:    10, // megabytes
		MaxBackups: 7,
		MaxAge:     28,   //days
		Compress:   true, // disabled by default
	}

	multi := io.MultiWriter(os.Stdout, logFile)
	log.Logger = zerolog.New(multi).With().Timestamp().Logger()
}
