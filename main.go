package main

import (
    "log"

    tea "github.com/charmbracelet/bubbletea"
    "github.com/Benji377/tooka/internal/app"
)

func main() {
	p := tea.NewProgram(app.InitialModel(), tea.WithAltScreen())
	if _, err := p.Run(); err != nil {
		log.Fatalf("Error running Tooka: %v", err)
	}
}
