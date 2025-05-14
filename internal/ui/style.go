package ui

import "github.com/charmbracelet/lipgloss"

var headerStyle = lipgloss.NewStyle().
	Foreground(lipgloss.Color("#00FFCC")).
	Background(lipgloss.Color("#222222")).
	Padding(1, 0).
	Align(lipgloss.Center).
	Bold(true)

var dividerStyle = lipgloss.NewStyle().
	Foreground(lipgloss.Color("#444444"))

var sortInfoStyle = lipgloss.NewStyle().
	Foreground(lipgloss.Color("241")).
	Italic(true).
	MarginBottom(1)

var helpStyle = lipgloss.NewStyle().
	Foreground(lipgloss.Color("240")).
	Padding(0, 1).
	Align(lipgloss.Center)
