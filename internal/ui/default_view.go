package ui

import (
	"fmt"
	"strings"

	"github.com/Benji377/tooka/internal/core"
	"github.com/charmbracelet/lipgloss"
)

func (m *model) defaultView() string {
	tasks := m.taskManager.List()
	if m.sortDir == Desc {
		// simple reversal; implement full sort logic later if needed
		reversed := make([]core.Task, len(tasks))
		for i := range tasks {
			reversed[i] = tasks[len(tasks)-1-i]
		}
		tasks = reversed
	}

	header := headerStyle.Render("Tooka")
	divider := dividerStyle.Render(strings.Repeat("─", m.width))
	footer := helpStyle.Render("a: Add  e: Edit  r: Remove  s: Sort  space: Toggle complete")

	leftWidth := m.width / 2
	rightWidth := m.width - leftWidth - 1 // -1 for the vertical divider

	// Sorting info
	sortInfo := sortInfoStyle.Render(fmt.Sprintf("Sorting: %s | Direction: %s", m.sortBy, m.sortDir))

	// Left pane (tasks)
	var taskList strings.Builder
	taskList.WriteString(sortInfo + "\n")

	for i, task := range tasks {
		cursor := " "
		if i == m.cursor {
			cursor = ">"
		}
		checkbox := "[ ]"
		if task.Completed {
			checkbox = "[x]"
		}
		title := truncate(task.Title, leftWidth-30)
		due := task.DueDate.Format("2006-01-02")
		line := fmt.Sprintf("%s %s %-20s %s\n", cursor, checkbox, title, lipgloss.NewStyle().Align(lipgloss.Right).Width(10).Render(due))
		taskList.WriteString(line)
	}

	leftPane := lipgloss.NewStyle().Width(leftWidth).Render(taskList.String())

	// Right pane (task detail)
	var detail string
	if m.cursor < len(tasks) {
		t := tasks[m.cursor]
		detail = fmt.Sprintf(
			"Title: %s\nDue: %s\nPriority: %s\nState: %s\n\n%s",
			t.Title,
			t.DueDate.Format("2006-01-02"),
			t.Priority,
			boolToState(t.Completed),
			t.Description,
		)
	}
	rightPane := lipgloss.NewStyle().Width(rightWidth).PaddingLeft(1).Render(detail)

	// Combine panes
	body := lipgloss.JoinHorizontal(lipgloss.Top,
		leftPane,
		lipgloss.NewStyle().Width(1).Foreground(lipgloss.Color("240")).Render("│"),
		rightPane,
	)

	return lipgloss.JoinVertical(lipgloss.Left,
		header,
		divider,
		body,
		divider,
		footer,
	)
}