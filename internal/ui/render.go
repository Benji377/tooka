package ui

import (
	"fmt"
	"strings"

	"github.com/Benji377/tooka/internal/core"
)

func RenderTaskList(tasks []core.Task) string {
	if len(tasks) == 0 {
		return ErrorStyle.Render("No tasks found.")
	}

	var b strings.Builder
	for _, task := range tasks {
		status := PendingStyle.Render("✗")
		if task.Completed {
			status = CompletedStyle.Render("✓")
		}

		priority := stylePriority(task.Priority)

		line := fmt.Sprintf("%s [%s] %s - %s\n",
			IDStyle.Render(fmt.Sprintf("#%d", task.ID)),
			status,
			TitleStyle.Render(task.Title),
			priority,
		)
		b.WriteString(line)
	}
	return b.String()
}

func RenderTaskDetails(task core.Task) string {
	var b strings.Builder

	b.WriteString(TitleStyle.Render(task.Title) + "\n")
	b.WriteString(fmt.Sprintf("%s %s\n", LabelStyle.Render("ID:"), IDStyle.Render(fmt.Sprintf("%d", task.ID))))
	b.WriteString(fmt.Sprintf("%s %s\n", LabelStyle.Render("Status:"), statusText(task.Completed)))
	b.WriteString(fmt.Sprintf("%s %s\n", LabelStyle.Render("Due Date:"), ValueStyle.Render(task.DueDate.Format("2006-01-02"))))
	b.WriteString(fmt.Sprintf("%s %s\n", LabelStyle.Render("Priority:"), stylePriority(task.Priority)))
	b.WriteString(fmt.Sprintf("%s\n%s\n", LabelStyle.Render("Description:"), ValueStyle.Render(task.Description)))

	return b.String()
}

func statusText(done bool) string {
	if done {
		return CompletedStyle.Render("Completed")
	}
	return PendingStyle.Render("Pending")
}

func stylePriority(p core.Priority) string {
	switch p {
	case core.Low:
		return PriorityLowStyle.Render("Low")
	case core.Medium:
		return PriorityMediumStyle.Render("Medium")
	case core.Severe:
		return PrioritySevereStyle.Render("Severe")
	default:
		return ValueStyle.Render("Unknown")
	}
}
