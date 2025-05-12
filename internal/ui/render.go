package ui

import (
	"fmt"
	"github.com/Benji377/tooka/internal/modules"
	"strings"
)

func RenderModuleInfo(info modules.ModuleInfo) string {
	content := fmt.Sprintf(
		"%s\n\n%s\n\n%s",
		HeaderStyle.Render("Module: "+info.Name),
		LabelStyle.Render("Description:")+"\n"+ValueStyle.Render(info.Description),
		LabelStyle.Render("Config Help:")+"\n"+ValueStyle.Render(info.ConfigHelp),
	)

	// Ensure we recalculate width in case terminal is resized
	width := TerminalWidth() - 4
	return BoxStyle.Width(width).Render(content)
}

func RenderTable(headers []string, rows [][]string) string {
	// Build header row
	var out strings.Builder

	colWidths := make([]int, len(headers))
	for i, h := range headers {
		colWidths[i] = len(h)
	}
	for _, row := range rows {
		for i, cell := range row {
			if len(cell) > colWidths[i] {
				colWidths[i] = len(cell)
			}
		}
	}

	// Header
	for i, h := range headers {
		out.WriteString(TableHeaderStyle.Render(fmt.Sprintf("%-*s", colWidths[i]+2, h)))
	}
	out.WriteString("\n")
	out.WriteString(strings.Repeat("─", sum(colWidths)+3*len(colWidths)))
	out.WriteString("\n")

	// Rows
	for _, row := range rows {
		for i, cell := range row {
			out.WriteString(TableRowStyle.Render(fmt.Sprintf("%-*s", colWidths[i]+2, cell)))
		}
		out.WriteString("\n")
	}

	return out.String()
}

func sum(arr []int) int {
	s := 0
	for _, n := range arr {
		s += n
	}
	return s
}
