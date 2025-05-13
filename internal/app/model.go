package app

import (
	"sort"
	"strings"

	"github.com/Benji377/tooka/internal/storage"
	"github.com/Benji377/tooka/internal/task"
	"github.com/charmbracelet/bubbles/textinput"
)

type SortMode int

const (
	SortByDueDate SortMode = iota
	SortByName
	SortByPriority
)

type model struct {
	tasks         []task.Task
	inputs        []textinput.Model
	editingInputs []textinput.Model
	adding        bool
	editing       bool
	inputIndex    int
	cursor        int
	viewingTask   bool
	viewedTask    task.Task
	sortMode      SortMode
	width         int
	height        int
	offset        int
}

var placeholders = []string{
	"Title",
	"Description",
	"Due Date (YYYY-MM-DD HH:MM)",
	"Priority (low, medium, severe)",
}

func InitialModel() model {
	// Load tasks from storage
	tasks, err := storage.LoadTasks()
	if err != nil {
		// Handle error (optional: could log it)
		panic("Error loading tasks: " + err.Error())
	}

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

	// Return the model with loaded tasks
	return model{
		tasks:         tasks,
		inputs:        createInputs(),
		editingInputs: createInputs(),
		inputIndex:    0,
		cursor:        0,
	}
}

func (m model) View() string {
	// If we're adding or editing a task, show the input form
	if m.adding || m.editing {
		return viewAddingOrEditing(m)
	}

	// If we're viewing a task's details, show the task details
	if m.viewingTask {
		return viewTaskDetails(m)
	}

	// Otherwise, show the task list
	return viewTaskList(m)
}

func (m *model) sortTasks() {
	switch m.sortMode {
	case SortByName:
		sort.Slice(m.tasks, func(i, j int) bool {
			return strings.ToLower(m.tasks[i].Title) < strings.ToLower(m.tasks[j].Title)
		})
	case SortByPriority:
		sort.Slice(m.tasks, func(i, j int) bool {
			return m.tasks[i].Priority > m.tasks[j].Priority
		})
	case SortByDueDate:
		fallthrough
	default:
		sort.Slice(m.tasks, func(i, j int) bool {
			return m.tasks[i].DueDate.Before(m.tasks[j].DueDate)
		})
	}
}
