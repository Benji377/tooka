package cmd

import (
	"fmt"
	"strings"

	"github.com/Benji377/tooka/internal/core"
	"github.com/Benji377/tooka/internal/modules"
	"github.com/Benji377/tooka/internal/ui"
	"github.com/spf13/cobra"
)

var moduleInfoCmd = &cobra.Command{
	Use:   "info <module-name>",
	Short: "Show info about a module type",
	Long:  `Retrieves metadata about a module type`,
	Args:  cobra.ExactArgs(1),
	Run: func(cmd *cobra.Command, args []string) {
		moduleName := strings.ToLower(args[0])
		core.Log.Info().Str("module", moduleName).Msg("Retrieving module information")

		info, ok := modules.GetModuleInfo(moduleName)
		if !ok {
			fmt.Println(ui.ErrorStyle.Render("❌ Unknown module: " + moduleName))
			core.Log.Warn().Str("module", moduleName).Msg("Unknown module requested")
			return
		}

		fmt.Println(ui.RenderModuleInfo(info))
	},
}
