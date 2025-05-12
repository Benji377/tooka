package cmd

import (
	"fmt"
	"strings"

	"github.com/Benji377/tooka/internal/modules"
	"github.com/charmbracelet/lipgloss"
	"github.com/spf13/cobra"
)

var (
	titleStyle   = lipgloss.NewStyle().Bold(true).Foreground(lipgloss.Color("212"))
	labelStyle   = lipgloss.NewStyle().Bold(true).Foreground(lipgloss.Color("99"))
	contentStyle = lipgloss.NewStyle().Foreground(lipgloss.Color("250"))
	boxBorder    = lipgloss.DoubleBorder()
	boxStyle     = lipgloss.NewStyle().Border(boxBorder).Padding(1, 2).BorderForeground(lipgloss.Color("63"))
)

var moduleInfoCmd = &cobra.Command{
	Use:   "info <module-name>",
	Short: "Show info about a module type",
	Args:  cobra.ExactArgs(1),
	Run: func(cmd *cobra.Command, args []string) {
		name := strings.ToLower(args[0])
		info, ok := modules.GetModuleInfo(name)
		if !ok {
			fmt.Println(lipgloss.NewStyle().Foreground(lipgloss.Color("9")).Render("❌ Unknown module: " + name))
			return
		}

		title := titleStyle.Render(fmt.Sprintf("Module: %s", info.Name))
		description := fmt.Sprintf("%s\n%s", labelStyle.Render("Description:"), contentStyle.Render(info.Description))
		config := fmt.Sprintf("%s\n%s", labelStyle.Render("Config Help:"), contentStyle.Render(info.ConfigHelp))

		box := boxStyle.Render(fmt.Sprintf("%s\n\n%s\n\n%s", title, description, config))
		fmt.Println(box)
	},
}
