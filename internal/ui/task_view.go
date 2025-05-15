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
		Align(lipgloss.Left).
		Background(baseBg) // Ensure background is set here

	// Consistent base style for all internal lines
	lineStyle := lipgloss.NewStyle().
		Width(formWidth).
		Background(baseBg).
		Foreground(baseFg)

	var lines []string

	// Title
	lines = append(lines, BigTitle.Width(formWidth).Render(title))
	lines = append(lines, "") // blank line

	// Inputs
	for i, input := range inputs {
		label := LabelStyle.Width(formWidth).Render(placeholders[i])
		inputLine := lineStyle.Render(input.View())

		lines = append(lines, label)
		lines = append(lines, inputLine)
		lines = append(lines, "") // spacing after input
	}

	// Footer
	footer := FooterStyle.Width(formWidth).Render("[Tab] Next • [Shift+Tab] Prev • [Enter] Confirm • [Esc] Cancel")
	lines = append(lines, footer)

	// Join all and render inside form container
	content := strings.Join(lines, "\n")
	formView := formContainer.Render(content)

	return lipgloss.Place(
		m.width, m.height,
		lipgloss.Center, lipgloss.Center,
		formView,
	)
}
