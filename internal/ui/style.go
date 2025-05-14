package ui

import "github.com/charmbracelet/lipgloss"

// Base colors
var (
	baseFg = lipgloss.Color("#f8f8f2")
	baseBg = lipgloss.Color("#282a36")
	accent = lipgloss.Color("#bd93f9")
	green  = lipgloss.Color("#50fa7b")
	red    = lipgloss.Color("#ff5555")
	yellow = lipgloss.Color("#f1fa8c")
)

// General styles
var (
	AppStyle = lipgloss.NewStyle().
			Padding(1, 2).
			Background(baseBg).
			Foreground(baseFg)

	TitleStyle = lipgloss.NewStyle().
			Bold(true).
			Foreground(accent).
			Align(lipgloss.Center).
			Background(baseBg)

	BigTitle = TitleStyle.
			Height(2)

	LabelStyle = lipgloss.NewStyle().
			Bold(true).
			Foreground(yellow)

	FooterStyle = lipgloss.NewStyle().
			Foreground(accent).
			Align(lipgloss.Center).
			MarginTop(1)

	TaskStatusDone = lipgloss.NewStyle().
			Foreground(green)

	TaskStatusTodo = lipgloss.NewStyle().
			Foreground(red)
)

// Pane and selection styles
var (
	SelectedTaskStyle = lipgloss.NewStyle().
				Foreground(baseFg).
				Background(lipgloss.Color("#44475a")).
				Bold(true)

	LeftPaneStyle = lipgloss.NewStyle().
			Background(lipgloss.Color("#1e1f29")).
			Padding(0, 1)

	RightPaneStyle = lipgloss.NewStyle().
			Background(lipgloss.Color("#1c1c1c")).
			Padding(0, 1)
)

// Status styles
var (
	OverdueStyle  = lipgloss.NewStyle().Foreground(red)
	UpcomingStyle = lipgloss.NewStyle().Foreground(yellow)
	FutureStyle   = lipgloss.NewStyle().Foreground(green)
)

// Miscellaneous styles
var (
	DividerGradient = lipgloss.NewStyle().
			Foreground(lipgloss.Color("#6272a4"))

	HeaderStyle = lipgloss.NewStyle().
			Foreground(lipgloss.Color("#00FFCC")).
			Background(lipgloss.Color("#222222")).
			Padding(1, 0).
			Align(lipgloss.Center).
			Bold(true)

	DividerStyle = lipgloss.NewStyle().
			Foreground(lipgloss.Color("#444444"))

	SortInfoStyle = lipgloss.NewStyle().
			Foreground(lipgloss.Color("241")).
			Italic(true).
			MarginBottom(1)

	HelpStyle = lipgloss.NewStyle().
			Foreground(lipgloss.Color("240")).
			Padding(0, 1).
			Align(lipgloss.Center)
)
