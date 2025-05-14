package ui

import (
	"strings"

	"github.com/charmbracelet/lipgloss"
)

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
