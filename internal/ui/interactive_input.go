package ui

import (
	"fmt"
	"time"

	"github.com/Benji377/tooka/internal/core"
	"github.com/Benji377/tooka/internal/shared"
	"github.com/charmbracelet/bubbles/textinput"
	tea "github.com/charmbracelet/bubbletea"
)

type taskFormModel struct {
	inputs    []textinput.Model
	focusIdx  int
	confirmed bool
	err       error
	done      chan struct{}
}

func initialTaskFormModel(existing *core.Task) taskFormModel {
	fields := []string{
		"Title",
		"Description",
		"Due Date (YYYY-MM-DD)",
		"Priority (0=Low, 1=Medium, 2=Severe)",
	}
	inputs := make([]textinput.Model, len(fields))

	var defaults []string
	if existing != nil {
		defaults = []string{
			existing.Title,
			existing.Description,
			existing.DueDate.Format("2006-01-02"),
			fmt.Sprintf("%d", existing.Priority),
		}
	} else {
		defaults = []string{"", "", "", ""}
	}

	for i, placeholder := range fields {
		ti := textinput.New()
		ti.Placeholder = placeholder
		ti.SetValue(defaults[i])
		ti.CharLimit = 100
		ti.Width = 40

		if i == 0 {
			ti.Focus()
		}

		inputs[i] = ti
	}

	shared.Log.Debug().Msg("Initialized task form for user input") // Debug log when initializing the form

	return taskFormModel{
		inputs: inputs,
		done:   make(chan struct{}),
	}
}

func (m taskFormModel) Init() tea.Cmd {
	return textinput.Blink
}

func (m taskFormModel) Update(msg tea.Msg) (tea.Model, tea.Cmd) {
	switch msg := msg.(type) {
	case tea.KeyMsg:
		switch msg.String() {
		case "ctrl+c", "esc":
			m.err = fmt.Errorf("cancelled")
			close(m.done)
			shared.Log.Info().Msg("Task creation/edit cancelled by user") // Info log for cancellation
			return m, tea.Quit

		case "enter":
			if m.focusIdx == len(m.inputs)-1 {
				m.confirmed = true
				close(m.done)
				shared.Log.Info().Msg("Task confirmed by user") // Info log when user confirms task creation/edit
				return m, tea.Quit
			}
			m.focusIdx++

		case "up":
			if m.focusIdx > 0 {
				m.focusIdx--
			}
		case "down":
			if m.focusIdx < len(m.inputs)-1 {
				m.focusIdx++
			}
		}

		for i := range m.inputs {
			if i == m.focusIdx {
				m.inputs[i].Focus()
			} else {
				m.inputs[i].Blur()
			}
		}
	}

	cmds := make([]tea.Cmd, len(m.inputs))
	for i := range m.inputs {
		m.inputs[i], cmds[i] = m.inputs[i].Update(msg)
	}
	return m, tea.Batch(cmds...)
}

func (m taskFormModel) View() string {
	s := "\n📝 Add/Edit Task\n\n"
	for i := range m.inputs {
		cursor := "  "
		if i == m.focusIdx {
			cursor = "👉"
		}
		s += fmt.Sprintf("%s %s\n", cursor, m.inputs[i].View())
	}
	s += "\n↑ ↓ to move • ↵ to confirm • ESC to cancel\n"
	return s
}

func PromptForTask(existing *core.Task) (*core.Task, error) {
	shared.Log.Debug().Msg("Prompting user for task input") // Debug log when prompting for task input
	m := initialTaskFormModel(existing)
	p := tea.NewProgram(m)
	finalModel, err := p.Run()
	if err != nil {
		shared.Log.Error().Err(err).Msg("Error running task input program") // Error log for running the task input program
		return nil, err
	}

	form := finalModel.(taskFormModel)
	if form.err != nil {
		shared.Log.Error().Err(form.err).Msg("Error in task form input") // Error log for form error
		return nil, form.err
	}
	if !form.confirmed {
		shared.Log.Info().Msg("Task input cancelled") // Info log for cancelled input
		return nil, fmt.Errorf("input cancelled")
	}

	title := form.inputs[0].Value()
	desc := form.inputs[1].Value()
	dueRaw := form.inputs[2].Value()
	prioRaw := form.inputs[3].Value()

	shared.Log.Debug().Str("title", title).Str("description", desc).Str("due_date", dueRaw).Str("priority", prioRaw).Msg("Task form details") // Debug log for collected input

	if title == "" {
		return nil, fmt.Errorf("title is required")
	}

	due, err := time.Parse("2006-01-02", dueRaw)
	if err != nil {
		shared.Log.Error().Err(err).Str("due_raw", dueRaw).Msg("Invalid due date") // Error log for invalid due date
		return nil, fmt.Errorf("invalid due date")
	}

	priority := 0
	if prioRaw != "" {
		if _, err := fmt.Sscanf(prioRaw, "%d", &priority); err != nil {
			shared.Log.Error().Err(err).Str("prio_raw", prioRaw).Msg("Invalid priority") // Error log for invalid priority
			return nil, fmt.Errorf("invalid priority")
		}
	}

	task := &core.Task{
		Title:       title,
		Description: desc,
		DueDate:     due,
		Priority:    core.Priority(priority),
		Completed:   existing != nil && existing.Completed,
	}

	shared.Log.Info().Int("task_id", task.ID).Str("title", task.Title).Msg("Created or edited task") // Info log for task creation/edit
	return task, nil

}
