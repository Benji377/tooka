package app

import (
	"fmt"
	"strings"
)

func viewTaskList(m model) string {
	var b strings.Builder
	b.WriteString(TitleStyle.Width(m.width).Render("📋 Your Tasks") + "\n")

	for i, t := range m.tasks {
		card := fmt.Sprintf("%s %s\nDue: %s", 
			map[bool]string{true: "✓", false: "✗"}[t.Completed],
			t.Title,
			t.DueDate.Format("2006-01-02"))

		b.WriteString(TaskCardStyle(t, i == m.cursor).Render(card) + "\n")
	}

	b.WriteString(FooterStyle.Width(m.width).Render("[↑/↓] Navigate • [Enter] View • [a] Add • [d] Delete • [q] Quit"))
	return AppStyle.Width(m.width).Render(b.String())
}

func viewTaskDetails(m model) string {
	t := m.viewedTask
	content := BigTitle.Render("🔍 " + t.Title) + "\n" +
		LabelStyle.Render("Description: ") + t.Description + "\n" +
		LabelStyle.Render("Due: ") + t.DueDate.Format("2006-01-02") + "\n" +
		LabelStyle.Render("Created: ") + t.CreatedAt.Format("2006-01-02 15:04") + "\n" +
		LabelStyle.Render("Status: ") + func() string {
			if t.Completed {
				return TaskStatusDone.Render("✓ Completed")
			}
			return TaskStatusTodo.Render("✗ Incomplete")
		}() + "\n\n"

	content += FooterStyle.Render("[Space] Toggle Done • [e] Edit • [Esc] Back")
	return AppStyle.Width(m.width).Render(content)
}

func viewAddingOrEditing(m model) string {
	var b strings.Builder
	title := "➕ Add New Task"
	inputs := m.inputs
	if m.editing {
		title = "✏️ Edit Task"
		inputs = m.editingInputs
	}

	b.WriteString(TitleStyle.Width(m.width).Render(title) + "\n\n")
	for i, input := range inputs {
		b.WriteString(LabelStyle.Render(placeholders[i]) + "\n")
		b.WriteString(input.View() + "\n\n")
	}

	b.WriteString(FooterStyle.Width(m.width).Render("[Tab] Next • [Shift+Tab] Prev • [Enter] Confirm • [Esc] Cancel"))
	return AppStyle.Width(m.width).Render(b.String())
}
