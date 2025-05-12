package ui

import (
	"os"

	"github.com/charmbracelet/lipgloss"
	"golang.org/x/term"
)

// TerminalWidth returns the current terminal width (fallbacks to 80)
func TerminalWidth() int {
	width, _, err := term.GetSize(int(os.Stdout.Fd()))
	if err != nil {
		return 80
	}
	return width
}

// Global Styles
var (
	HeaderStyle = lipgloss.NewStyle().
			Bold(true).
			Foreground(lipgloss.Color("205"))

	LabelStyle = lipgloss.NewStyle().
			Bold(true).
			Foreground(lipgloss.Color("99"))

	ValueStyle = lipgloss.NewStyle().
			Foreground(lipgloss.Color("250"))

	ErrorStyle = lipgloss.NewStyle().
			Foreground(lipgloss.Color("9"))

	SuccessStyle = lipgloss.NewStyle().
			Foreground(lipgloss.Color("10"))

	TableHeaderStyle = lipgloss.NewStyle().
				Bold(true).
				Foreground(lipgloss.Color("205"))

	TableRowStyle = lipgloss.NewStyle().
			Foreground(lipgloss.Color("250"))

	BoxStyle = lipgloss.NewStyle().
			Border(lipgloss.DoubleBorder()).
			Padding(1, 2).
			BorderForeground(lipgloss.Color("63")).
			Width(TerminalWidth() - 4)
)
