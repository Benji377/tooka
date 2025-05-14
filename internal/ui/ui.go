package ui

import (
	"fmt"
	"strings"

	"github.com/Benji377/tooka/internal/core"
	tea "github.com/charmbracelet/bubbletea"
	"github.com/charmbracelet/lipgloss"
)

const minWidth = 80
const minHeight = 20

type sortField string
type sortDirection string

const (
	SortByTitle    sortField = "Title"
	SortByDueDate  sortField = "Due Date"
	SortByPriority sortField = "Priority"
	SortByState    sortField = "State"

	Asc  sortDirection = "Asc"
	Desc sortDirection = "Desc"
)

type model struct {
	width, height int
	taskManager   *core.TaskManager
	cursor        int
	sortBy        sortField
	sortDir       sortDirection
	errMsg        string
}

func New(taskManager *core.TaskManager) tea.Model {
	return &model{
		taskManager: taskManager,
		sortBy:      SortByDueDate,
		sortDir:     Asc,
	}
}

func (m *model) Init() tea.Cmd {
	return nil
}

func (m *model) Update(msg tea.Msg) (tea.Model, tea.Cmd) {
	switch msg := msg.(type) {

	case tea.WindowSizeMsg:
		m.width = msg.Width
		m.height = msg.Height
		if m.width < minWidth || m.height < minHeight {
			m.errMsg = fmt.Sprintf("Terminal too small. Resize to at least %dx%d", minWidth, minHeight)
		} else {
			m.errMsg = ""
		}
	case tea.KeyMsg:
		switch msg.String() {
		case "ctrl+c", "q":
			return m, tea.Quit
		case "up", "k":
			if m.cursor > 0 {
				m.cursor--
			}
		case "down", "j":
			if m.cursor < len(m.taskManager.List())-1 {
				m.cursor++
			}
		case "space":
			tasks := m.taskManager.List()
			if len(tasks) > 0 {
				_ = m.taskManager.ToggleComplete(tasks[m.cursor].ID)
			}
		case "s":
			m.toggleSort()
		}
	}
	return m, nil
}

func (m *model) View() string {
	if m.errMsg != "" {
		return lipgloss.NewStyle().Align(lipgloss.Center).Height(m.height).Render(m.errMsg)
	}

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

func (m *model) toggleSort() {
	switch m.sortBy {
	case SortByTitle:
		m.sortBy = SortByDueDate
	case SortByDueDate:
		m.sortBy = SortByPriority
	case SortByPriority:
		m.sortBy = SortByState
	case SortByState:
		m.sortBy = SortByTitle
	}
	if m.sortDir == Asc {
		m.sortDir = Desc
	} else {
		m.sortDir = Asc
	}
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
