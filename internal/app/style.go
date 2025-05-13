package app

import (
    "time"

    "github.com/charmbracelet/lipgloss"
    "github.com/Benji377/tooka/internal/task"
)

var (
    baseFg = lipgloss.Color("#f8f8f2")
    baseBg = lipgloss.Color("#282a36")
    accent = lipgloss.Color("#bd93f9")
    green  = lipgloss.Color("#50fa7b")
    red    = lipgloss.Color("#ff5555")
    yellow = lipgloss.Color("#f1fa8c")

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

func TaskCardStyle(t task.Task, selected bool) lipgloss.Style {
    borderColor := green
    today := time.Now().Truncate(24 * time.Hour)
    due := t.DueDate.Truncate(24 * time.Hour)

    switch {
    case due.Before(today):
        borderColor = red
    case due.Equal(today):
        borderColor = yellow
    }

    style := lipgloss.NewStyle().
        BorderStyle(lipgloss.RoundedBorder()).
        BorderForeground(borderColor).
        Padding(0, 2).
        Margin(1, 0).
        Width(50)

    if selected {
        style = style.Copy().BorderForeground(accent).Bold(true)
    }

    return style
}
