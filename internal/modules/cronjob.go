package modules

import (
	"fmt"
	"os/exec"
	"github.com/go-co-op/gocron"
	"log"
	"time"
)

type CronJobModule struct {
	Schedule string
	Command  string
}

func NewCronJobModule(config map[string]any) (Module, error) {
	schedule, okSchedule := config["schedule"].(string)
	command, okCommand := config["command"].(string)

	if !okSchedule || schedule == "" || !okCommand || command == "" {
		return nil, fmt.Errorf("both 'schedule' (cron-style) and 'command' must be provided in cronjob module config")
	}

	return &CronJobModule{
		Schedule: schedule,
		Command:  command,
	}, nil
}

func (m *CronJobModule) Run() string {
	// Create a new scheduler
	scheduler := gocron.NewScheduler(time.UTC)

	// Define the cron job based on the provided schedule
	_, err := scheduler.Cron(m.Schedule).Do(func() {
		// Execute the command as scheduled
		cmd := exec.Command("sh", "-c", m.Command)
		out, err := cmd.CombinedOutput()
		if err != nil {
			log.Printf("Error executing cron command: %v\n", err)
		} else {
			log.Printf("Cronjob executed successfully: %s\n", out)
		}
	})
	if err != nil {
		return fmt.Sprintf("Error setting up cron job: %s", err.Error())
	}

	// Start the scheduler in the background
	go scheduler.StartAsync()

	// For demonstration, we let it run for a fixed period
	// (For real-world usage, you would probably want the scheduler running indefinitely)
	time.Sleep(10 * time.Second)

	// Stop the scheduler after the test runs
	scheduler.Stop()

	return "Cron job executed (check logs)"
}

func init() {
	RegisterModule(ModuleInfo{
		Name:        "cronjob",
		Description: "Executes a command at scheduled intervals using cron syntax with gocron",
		ConfigHelp:  "Required: 'schedule' (string), 'command' (string)",
		Constructor: NewCronJobModule,
	})
}
