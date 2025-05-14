package ui

import "github.com/charmbracelet/lipgloss"

var (
	TitleStyle = lipgloss.NewStyle().
			Bold(true).
			Foreground(lipgloss.Color("#7D56F4")).
			MarginBottom(1)

	LabelStyle = lipgloss.NewStyle().
			Bold(true).
			Foreground(lipgloss.Color("#FFA500"))

	ValueStyle = lipgloss.NewStyle().
			Foreground(lipgloss.Color("#FFFFFF"))

	IDStyle = lipgloss.NewStyle().
		Foreground(lipgloss.Color("#00FFFF")).
		Bold(true)

	CompletedStyle = lipgloss.NewStyle().
			Foreground(lipgloss.Color("#00FF00")).
			Bold(true)

	PendingStyle = lipgloss.NewStyle().
			Foreground(lipgloss.Color("#0000FF")).
			Bold(true)

	DueTodayStyle = lipgloss.NewStyle().
			Foreground(lipgloss.Color("#FFD700")).
			Bold(true)

	OverdueStyle = lipgloss.NewStyle().
			Foreground(lipgloss.Color("#FF0000")).
			Bold(true)

	PriorityLowStyle = lipgloss.NewStyle().
				Foreground(lipgloss.Color("#00CED1"))

	PriorityMediumStyle = lipgloss.NewStyle().
				Foreground(lipgloss.Color("#FFD700"))

	PrioritySevereStyle = lipgloss.NewStyle().
				Foreground(lipgloss.Color("#FF4500"))

	ErrorStyle = lipgloss.NewStyle().
			Foreground(lipgloss.Color("#FF5555")).
			Bold(true)
)
