package ui

import (
	"fmt"
	"os"
	"strings"
	"time"

	"github.com/Benji377/tooka/internal/core"
	"github.com/charmbracelet/lipgloss"
	"golang.org/x/term"
)

func RenderTaskList(tasks []core.Task) string {
	if len(tasks) == 0 {
		return ErrorStyle.Render("No tasks found.")
	}

	// Terminal width detection using golang.org/x/term
	width, _, err := term.GetSize(int(os.Stdout.Fd()))
	if err != nil || width <= 0 {
		width = 100
	}
	cardWidth := max(width-10, 60)

	var b strings.Builder
	for _, task := range tasks {
		// Status styles
		var statusStyle lipgloss.Style
		now := time.Now()
		due := task.DueDate

		var statusText string
		if task.Completed {
			statusStyle = CompletedStyle
			statusText = "Completed"
		} else if due.Year() == now.Year() && due.Month() == now.Month() && due.Day() == now.Day() {
			statusStyle = DueTodayStyle
			statusText = "Due Today"
		} else if due.Before(now) {
			statusStyle = OverdueStyle
			statusText = "Overdue"
		} else {
			statusStyle = PendingStyle
			statusText = "Todo"
		}

		// Format task description (first 100 chars with ellipsis)
		description := task.Description
		if len(description) > 100 {
			description = description[:100] + "..."
		}

		// Format priority
		priority := stylePriority(task.Priority)

		// Format due date (just date, no time)
		dueDate := task.DueDate.Format("2006-01-02")

		// Define the light grey color for the border and task ID
		borderStyle := lipgloss.NewStyle().Foreground(lipgloss.Color("#D3D3D3"))

		// Define the text styles for title (white for dark background, black for light background)
		titleStyle := lipgloss.NewStyle().Foreground(lipgloss.Color("#FFFFFF"))

		// Card styling with padding and full width
		cardStyle := lipgloss.NewStyle().
			Border(lipgloss.NormalBorder(), true).
			BorderForeground(lipgloss.Color("#D3D3D3")).
			Width(cardWidth).
			Padding(1, 2).
			Align(lipgloss.Left)

		// Header section with task ID and status
		header := fmt.Sprintf("%s [%s] %s",
			borderStyle.Render(fmt.Sprintf("#%d", task.ID)),
			statusStyle.Render(statusText),
			titleStyle.Render(task.Title))

		// Description and other task details inside the card
		body := fmt.Sprintf("%s\n----------\n%s\n----------\nDue:      %s\nPriority: %s",
			borderStyle.Render(header),
			ValueStyle.Render(description),
			dueDate,
			priority,
		)

		// Combining card header, body, and styling
		card := cardStyle.Render(body)
		b.WriteString(card + "\n")
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
