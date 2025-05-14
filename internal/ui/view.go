// view.go

package ui

import (
	"github.com/charmbracelet/lipgloss"
)

const minWidth = 80
const minHeight = 20

func (m *model) View() string {
	if m.errMsg != "" {
		return lipgloss.NewStyle().Align(lipgloss.Center).Height(m.height).Render(m.errMsg)
	}

	if m.adding || m.editing {
		return viewAddingOrEditing(*m)
	}
	return m.defaultView()
}

func truncate(s string, max int) string {
	if max <= 3 {
		if max <= 0 {
			return ""
		}
		return s[:max]
	}
	if len(s) > max {
		return s[:max-3] + "..."
	}
	return s
}

func boolToState(b bool) string {
	if b {
		return "Completed"
	}
	return "Not Completed"
}
