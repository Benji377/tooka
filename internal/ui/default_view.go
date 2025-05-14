package ui

import (
	"fmt"
	"strings"
	"time"

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

	header := TitleStyle.Width(m.width).Render(`
  _____           _         
 |_   _|__   ___ | | ____ _ 
   | |/ _ \ / _ \| |/ / _' |
   | | (_) | (_) |   < (_| |
   |_|\___/ \___/|_|\_\__,_|
	`)
	divider := DividerGradient.Render(strings.Repeat("─", m.width))
	footer := HelpStyle.Render("a: Add  e: Edit  r: Remove  s: Sort  d: Sort Direction  space: Toggle complete")

	leftWidth := m.width / 2
	rightWidth := m.width - leftWidth - 1 // -1 for the vertical divider

	// Sorting info
	sortInfo := SortInfoStyle.Render(fmt.Sprintf("Sorting: %s | Direction: %s", m.sortBy, m.sortDir))

	// Left pane (tasks)
	var taskList strings.Builder
	taskList.WriteString(sortInfo + "\n")

	now := time.Now()

	for i, task := range tasks {
		cursor := " "
		lineStyle := lipgloss.NewStyle()
		if i == m.cursor {
			cursor = ">"
			lineStyle = SelectedTaskStyle
		}

		checkbox := "[ ]"
		if task.Completed {
			checkbox = "[x]"
		}

		title := truncate(task.Title, leftWidth-30)

		dueStyle := FutureStyle
		if task.DueDate.Before(now) && !task.Completed {
			dueStyle = OverdueStyle
		} else if task.DueDate.Before(now.AddDate(0, 0, 7)) {
			dueStyle = UpcomingStyle
		}

		due := dueStyle.Render(task.DueDate.Format("2006-01-02"))
		line := fmt.Sprintf("%s %s %-20s %s", cursor, checkbox, title, due)

		// Render padded line to the full width
		padded := lipgloss.NewStyle().Width(leftWidth).Render(line)
		taskList.WriteString(lineStyle.Render(padded) + "\n")
	}

	leftPane := LeftPaneStyle.Width(leftWidth).Height(minHeight).Render(taskList.String())

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
	rightPane := RightPaneStyle.Width(rightWidth).Height(minHeight).Render(detail)

	var vDividerBuilder strings.Builder
	for i := 0; i < minHeight; i++ {
		vDividerBuilder.WriteString("│\n")
	}
	vDivider := lipgloss.NewStyle().
		Foreground(lipgloss.Color("240")).
		Render(strings.TrimRight(vDividerBuilder.String(), "\n"))


	// Combine panes
	body := lipgloss.JoinHorizontal(lipgloss.Top,
		leftPane,
		vDivider,
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
