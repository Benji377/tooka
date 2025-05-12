package core

import (
	"os"
	"path/filepath"
)

var Version string = "1.0.0"
var TasksDir = filepath.Join(os.Getenv("HOME"), ".tooka", "tasks")
var LogsDir = filepath.Join(os.Getenv("HOME"), ".tooka", "logs")
