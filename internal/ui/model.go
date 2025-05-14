package ui

import (
	"github.com/Benji377/tooka/internal/core"
	"github.com/charmbracelet/bubbles/textinput"
	tea "github.com/charmbracelet/bubbletea"
)

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
	inputs	      []textinput.Model
	editingInputs []textinput.Model
	adding        bool
	editing       bool
	inputIndex    int
	sortBy        sortField
	sortDir       sortDirection
	errMsg        string
}

var placeholders = []string{
	"Title",
	"Description",
	"Due Date (YYYY-MM-DD)",
	"Priority (low, medium, severe)",
}

func New(taskManager *core.TaskManager) tea.Model {
	createInputs := func() []textinput.Model {
		inputs := make([]textinput.Model, len(placeholders))
		for i := range placeholders {
			ti := textinput.New()
			ti.Placeholder = placeholders[i]
			ti.CharLimit = 100
			inputs[i] = ti
		}
		return inputs
	}

	return &model{
		taskManager: taskManager,
		inputs:        createInputs(),
		editingInputs: createInputs(),
		inputIndex:    0,
		cursor:        0,
		sortBy:      SortByDueDate,
		sortDir:     Asc,
	}
}

func (m *model) Init() tea.Cmd {
	return nil
}
