package cmd

import (
	"fmt"

	"github.com/Benji377/tooka/internal/modules"
	"github.com/Benji377/tooka/internal/ui"
	"github.com/spf13/cobra"
)

var moduleListCmd = &cobra.Command{
	Use:   "list",
	Short: "List available module types",
	Run: func(cmd *cobra.Command, args []string) {

		// Table Header
		rows := []string{"Name", "Description"}
		table := [][]string{}
		for _, name := range modules.GetRegisteredModules() {
			info, _ := modules.GetModuleInfo(name)
			table = append(table, []string{name, info.Description})
		}
		fmt.Println(ui.HeaderStyle.Render("Available Modules:\n"))
		fmt.Println(ui.RenderTable(rows, table))
	},
}
