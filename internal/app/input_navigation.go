package app

import "github.com/charmbracelet/bubbles/textinput"

func updateInputNavigation(inputs []textinput.Model, index int, forward bool) ([]textinput.Model, int) {
	inputs[index].Blur()
	if forward && index < len(inputs)-1 {
		index++
	} else if !forward && index > 0 {
		index--
	}
	inputs[index].Focus()
	return inputs, index
}
