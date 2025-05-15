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
	rightWidth := m.width - leftWidth - 1
	usableHeight := m.height - 12

	sortInfo := SortInfoStyle.Render(fmt.Sprintf("Sorting: %s | Direction: %s", m.sortBy, m.sortDir))

	// LEFT PANE: TASK LIST
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
		padded := lipgloss.NewStyle().Width(leftWidth).Render(line)
		taskList.WriteString(lineStyle.Render(padded) + "\n")
	}

	leftPane := LeftPaneStyle.
		Width(leftWidth).
		Height(usableHeight).
		Render(taskList.String())

	// RIGHT PANE: TASK DETAILS
	var rightPaneContent string
	bg := lipgloss.Color("#1c1c1c")
	fieldStyle := lipgloss.NewStyle().Foreground(baseFg).Background(bg)
	lineStyle := lipgloss.NewStyle().Width(rightWidth).Background(bg)

	if m.cursor < len(tasks) {
		t := tasks[m.cursor]
		var lines []string

		lines = append(lines, lineStyle.Render(LabelStyle.Background(bg).Render("Title: ")+fieldStyle.Render(t.Title)))
		lines = append(lines, lineStyle.Render(LabelStyle.Background(bg).Render("Due: ")+FutureStyle.Background(bg).Render(t.DueDate.Format("2006-01-02"))))
		lines = append(lines, lineStyle.Render(LabelStyle.Background(bg).Render("Priority: ")+fieldStyle.Render(fmt.Sprintf("%v", t.Priority))))
		stateStyle := TaskStatusTodo.Background(bg)
		if t.Completed {
			stateStyle = TaskStatusDone.Background(bg)
		}
		lines = append(lines, lineStyle.Render(LabelStyle.Background(bg).Render("State: ")+stateStyle.Render(boolToState(t.Completed))))

		if t.Description != "" {
			lines = append(lines, lineStyle.Render(" ")) // Spacer line
			lines = append(lines, lineStyle.Render(LabelStyle.Background(bg).Render("Description:")))
			descLines := strings.SplitSeq(t.Description, "\n")
			for l := range descLines {
				lines = append(lines, lineStyle.Render(fieldStyle.Render(l)))
			}
		}

		rightPaneContent = lipgloss.JoinVertical(lipgloss.Left, lines...)
	} else {
		rightPaneContent = lipgloss.NewStyle().
			Italic(true).
			Foreground(lipgloss.Color("240")).
			Width(rightWidth).
			Background(bg).
			Render("No task selected.")
	}

	rightPane := RightPaneStyle.
		Width(rightWidth).
		Height(usableHeight).
		Render(rightPaneContent)

	// DIVIDER
	var vDividerBuilder strings.Builder
	for range usableHeight {
		vDividerBuilder.WriteString("│\n")
	}
	vDivider := lipgloss.NewStyle().
		Foreground(lipgloss.Color("240")).
		Render(strings.TrimRight(vDividerBuilder.String(), "\n"))

	// FINAL LAYOUT
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
