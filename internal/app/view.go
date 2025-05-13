package app

import (
	"fmt"
	"strings"

	"github.com/Benji377/tooka/internal/task"
	"github.com/charmbracelet/lipgloss"
)

func viewTaskList(m model) string {
	var b strings.Builder

	sortLabel := map[SortMode]string{
		SortByDueDate:  "Sort: Due Date",
		SortByName:     "Sort: Name",
		SortByPriority: "Sort: Priority",
	}[m.sortMode]

	// Title with sort indicator
	header := lipgloss.JoinHorizontal(lipgloss.Top,
		TitleStyle.Width(m.width-len(sortLabel)-10).Render("📋 Your Tasks"),
		lipgloss.NewStyle().Align(lipgloss.Right).Width(len(sortLabel)).Render(sortLabel),
	)
	b.WriteString(header + "\n")

	incomplete := filterTasksByCompletion(m.tasks, false)
	completed := filterTasksByCompletion(m.tasks, true)

	// Determine how many lines fit in the terminal (minus header/footer space)
	usableHeight := m.height - 10 // Adjust this as needed based on padding
	totalIncomplete := len(incomplete)

	// Ensure cursor is always within offset view
	if m.cursor < m.offset {
		m.offset = m.cursor
	} else if m.cursor >= m.offset+usableHeight {
		m.offset = m.cursor - usableHeight + 1
	}

	// Clamp offset if too high
	if m.offset > totalIncomplete-usableHeight {
		m.offset = max(0, totalIncomplete-usableHeight)
	}

	// Render visible incomplete tasks
	if totalIncomplete > 0 {
		b.WriteString("\n📅 Incomplete Tasks:\n")
		end := min(m.offset+usableHeight, totalIncomplete)

		for i := m.offset; i < end; i++ {
			t := incomplete[i]
			selected := i == m.cursor
			content := fmt.Sprintf("✗ %s\nDue: %s\nPriority: %s",
				t.Title,
				t.DueDate.Format("2006-01-02"),
				t.Priority.String(),
			)

			card := TaskCardStyle(t, selected).MaxWidth(m.width - 4).Render(content)
			b.WriteString(card + "\n")
		}
	} else {
		b.WriteString("🎉 No incomplete tasks!\n")
	}

	// Render completed tasks below (not scrollable)
	if len(completed) > 0 {
		b.WriteString("\n✅ Completed Tasks:\n")
		for _, t := range completed {
			content := fmt.Sprintf("✓ %s\nDue: %s\nPriority: %s",
				t.Title,
				t.DueDate.Format("2006-01-02"),
				t.Priority.String(),
			)
			card := TaskCardStyle(t, false).MaxWidth(m.width - 4).Render(content)
			b.WriteString(card + "\n")
		}
	}

	// Footer instructions
	footer := FooterStyle.Width(m.width).Render("[↑/↓] Navigate • [Enter] View • [a] Add • [d] Delete • [s] Sort • [q] Quit")
	b.WriteString("\n" + footer)

	return AppStyle.Width(m.width).Render(b.String())
}

func viewTaskDetails(m model) string {
	t := m.tasks[m.cursor]

	content := fmt.Sprintf(
		"Title: %s\n\nDescription:\n%s\n\nDue: %s\nPriority: %s\nCompleted: %t\nCreated At: %s",
		t.Title,
		t.Description,
		t.DueDate.Format("2006-01-02"),
		t.Priority.String(),
		t.Completed,
		t.CreatedAt.Format("2006-01-02 15:04"),
	)

	card := lipgloss.NewStyle().
		BorderStyle(lipgloss.DoubleBorder()).
		BorderForeground(accent).
		Padding(1, 3).
		Width(min(80, m.width-4)).
		Render(content)

	return AppStyle.Width(m.width).Render(card + "\n\n" +
		FooterStyle.Render("[Esc] Back • [e] Edit • [d] Delete"))
}

func viewAddingOrEditing(m model) string {
	title := "➕ Add New Task"
	inputs := m.inputs
	if m.editing {
		title = "✏️ Edit Task"
		inputs = m.editingInputs
	}

	formWidth := min(70, m.width-4) // cap width, leave padding

	// Stylized container for the form
	formContainer := lipgloss.NewStyle().
		BorderStyle(lipgloss.NormalBorder()).
		BorderForeground(accent).
		Padding(1, 2).
		Width(formWidth).
		Align(lipgloss.Left)

	content := strings.Builder{}
	content.WriteString(BigTitle.Width(formWidth).Render(title) + "\n\n")

	for i, input := range inputs {
		label := LabelStyle.Render(placeholders[i])
		content.WriteString(label + "\n")
		content.WriteString(input.View() + "\n\n")
	}

	footer := FooterStyle.Width(formWidth).Render("[Tab] Next • [Shift+Tab] Prev • [Enter] Confirm • [Esc] Cancel")
	content.WriteString(footer)

	formView := formContainer.Render(content.String())
	return AppStyle.Width(m.width).Align(lipgloss.Center).Render(formView)
}

func filterTasksByCompletion(tasks []task.Task, completed bool) []task.Task {
	var result []task.Task
	for _, t := range tasks {
		if t.Completed == completed {
			result = append(result, t)
		}
	}
	return result
}
