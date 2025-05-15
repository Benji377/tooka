package ui

import "github.com/charmbracelet/lipgloss"

// Refined base colors
var (
	baseFg = lipgloss.Color("#dcdfe4") // softer white
	baseBg = lipgloss.Color("#1e1e2e") // modern dark
	accent = lipgloss.Color("#89b4fa") // soft blue
	green  = lipgloss.Color("#a6e3a1") // gentle green
	red    = lipgloss.Color("#f38ba8") // muted red
	yellow = lipgloss.Color("#f9e2af") // pastel yellow
	gray   = lipgloss.Color("#313244")
	lightGray = lipgloss.Color("#6c7086")
)

// App container style
var (
	AppStyle = lipgloss.NewStyle().
		Padding(1, 2).
		Background(baseBg).
		Foreground(baseFg)

	TitleStyle = lipgloss.NewStyle().
		Bold(true).
		Foreground(accent).
		Align(lipgloss.Center).
		MarginBottom(1)

	BigTitle = TitleStyle.Height(2)

	LabelStyle = lipgloss.NewStyle().
		Bold(true).
		Foreground(lightGray)

	FooterStyle = lipgloss.NewStyle().
		Foreground(lightGray).
		Align(lipgloss.Center).
		MarginTop(1).
		Italic(true)

	TaskStatusDone = lipgloss.NewStyle().Foreground(green)
	TaskStatusTodo = lipgloss.NewStyle().Foreground(red)
)

// Pane & selection
var (
	SelectedTaskStyle = lipgloss.NewStyle().
		Foreground(baseFg).
		Background(lipgloss.Color("#45475a")).
		Bold(true)

	LeftPaneStyle = lipgloss.NewStyle().
		Background(lipgloss.Color("#181825")).
		Padding(0, 1)

	RightPaneStyle = lipgloss.NewStyle().
		Background(lipgloss.Color("#11111b")).
		Padding(0, 1)
)

// Task status by date
var (
	OverdueStyle  = lipgloss.NewStyle().Foreground(red)
	UpcomingStyle = lipgloss.NewStyle().Foreground(yellow)
	FutureStyle   = lipgloss.NewStyle().Foreground(green)
)

// Misc styles
var (
	DividerGradient = lipgloss.NewStyle().
		Foreground(gray)

	HeaderStyle = lipgloss.NewStyle().
		Foreground(accent).
		Background(baseBg).
		Padding(1, 0).
		Align(lipgloss.Center).
		Bold(true)

	DividerStyle = lipgloss.NewStyle().
		Foreground(gray)

	SortInfoStyle = lipgloss.NewStyle().
		Foreground(lightGray).
		Italic(true).
		MarginBottom(1)

	HelpStyle = lipgloss.NewStyle().
		Foreground(lightGray).
		Padding(0, 1).
		Align(lipgloss.Center)
)
