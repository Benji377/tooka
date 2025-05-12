package cmd

import (
	"fmt"
	"sync"
	"time"

	"github.com/Benji377/tooka/internal/core"
	"github.com/Benji377/tooka/internal/ui"
	"github.com/briandowns/spinner"
	"github.com/spf13/cobra"
)

var (
	taskNames  []string
	outputPath string
	quiet      bool
)

var runCmd = &cobra.Command{
	Use:   "run [task-name...]",
	Short: "Run one or more tasks by name",
	Long:  "Executes one or more Tooka tasks by name, optionally writing output to a file and showing results.",
	Args:  cobra.MinimumNArgs(1),
	Run: func(cmd *cobra.Command, args []string) {
		core.Log.Info().Msg("[Tooka RUN] Running tasks: " + fmt.Sprintf("%v", args))
		taskNames = args
		var wg sync.WaitGroup
		results := make(chan runResult, len(taskNames))

		core.Log.Info().Msg("[Tooka RUN] Starting task execution")

		// Iterate over the task names and run them concurrently
		// We use a WaitGroup to wait for all tasks to finish
		for _, name := range taskNames {
			task, ok := taskManager.GetTask(name)
			core.Log.Info().Msg("[Tooka RUN] Loading task: " + name)
			if !ok {
				results <- runResult{name: name, err: fmt.Errorf("task not found")}
				core.Log.Warn().Msg("[Tooka RUN] Task not found: " + name)
				continue
			}

			wg.Add(1)
			core.Log.Info().Msg("[Tooka RUN] Running task: " + name)
			go func(t *core.Task) {
				defer wg.Done()
				s := spinner.New(spinner.CharSets[14], 100*time.Millisecond)
				if !quiet {
					s.Prefix = fmt.Sprintf("Running %s... ", t.Name)
					s.Start()
				}
				err := t.RunLive(outputPath, quiet)
				if !quiet {
					s.Stop()
				}
				results <- runResult{name: t.Name, err: err}
			}(task)
			core.Log.Info().Msg("[Tooka RUN] Task started: " + name)
		}

		wg.Wait()
		close(results)
		core.Log.Info().Msg("[Tooka RUN] All tasks completed")

		success := []string{}
		failed := []string{}
		for res := range results {
			if res.err != nil {
				failed = append(failed, fmt.Sprintf("%s: %v", res.name, res.err))
			} else {
				success = append(success, res.name)
			}
		}
		if !quiet {
			fmt.Println("\n" + ui.HeaderStyle.Render("Execution Summary"))
		}
		if len(success) > 0 && !quiet {
			fmt.Println(ui.SuccessStyle.Render("✅ Success:"))
			for _, name := range success {
				fmt.Println(" - " + name)
			}
		}
		if len(failed) > 0 {
			fmt.Println(ui.ErrorStyle.Render("❌ Failed:"))
			for _, fail := range failed {
				fmt.Println(" - " + fail)
			}
		}
	},
}

type runResult struct {
	name string
	err  error
}

func init() {
	runCmd.Flags().StringVarP(&outputPath, "output", "o", "", "Override output file path for all tasks")
	runCmd.Flags().BoolVarP(&quiet, "quiet", "q", false, "Suppress module output during execution")
	rootCmd.AddCommand(runCmd)
}
