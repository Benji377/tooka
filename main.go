package main

import (
	"log"

	"github.com/Benji377/tooka/internal/core"
	"github.com/Benji377/tooka/internal/shared"
	"github.com/Benji377/tooka/internal/ui"
	tea "github.com/charmbracelet/bubbletea"
)

func main() {
	// Initialize the logger
	shared.InitLogger()
	log.SetFlags(log.Ldate | log.Ltime | log.Lshortfile)
	manager, err := core.NewTaskManager()
	if err != nil {
		log.Fatal(err)
	}

	if _, err := tea.NewProgram(ui.New(manager)).Run(); err != nil {
		log.Fatal(err)
	}
}
