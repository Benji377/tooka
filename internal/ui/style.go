package ui

import "github.com/charmbracelet/lipgloss"

var (
	baseFg = lipgloss.Color("#f8f8f2")
	baseBg = lipgloss.Color("#282a36")
	accent = lipgloss.Color("#bd93f9")
	green  = lipgloss.Color("#50fa7b")
	red    = lipgloss.Color("#ff5555")
	yellow = lipgloss.Color("#f1fa8c")
)

var (
	AppStyle = lipgloss.NewStyle().
			Padding(1, 2).
			Background(baseBg).
			Foreground(baseFg)

	TitleStyle = lipgloss.NewStyle().
			Bold(true).
			Foreground(accent).
			MarginBottom(1).
			Align(lipgloss.Center).
			Height(1).
			Faint(false).
			Italic(false).
			Background(baseBg).
			Padding(0, 0).
			MarginTop(1).
			MarginBottom(1).
			Inherit(AppStyle).
			Width(80)

	BigTitle = TitleStyle.Copy().Bold(true).Foreground(accent).Height(2).Align(lipgloss.Center)

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
