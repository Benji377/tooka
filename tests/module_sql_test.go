package tests

import (
	"os"
	"os/exec"
	"strings"
	"testing"
)

func TestSQLModule(t *testing.T) {
	dbPath := "test.db"

	// Prepare SQLite test DB
	if err := os.WriteFile(dbPath, nil, 0644); err != nil {
		t.Fatalf("Failed to create test DB file: %v", err)
	}
	defer func() {
		if err := os.Remove(dbPath); err != nil {
			t.Fatalf("Failed to remove test DB file: %v", err)
		}
	}()

	createSchema := exec.Command("sqlite3", dbPath, "CREATE TABLE items (id INTEGER PRIMARY KEY, name TEXT); INSERT INTO items (name) VALUES ('apple');")
	if err := createSchema.Run(); err != nil {
		t.Fatalf("Failed to initialize SQLite DB: %v", err)
	}

	taskFile := "sqlite_task.json"
	taskJSON := `{
		"name": "sqlite-task",
		"description": "Test SQLite query",
		"modules": [
			{
				"name": "sql",
				"config": {
					"db": "test.db",
					"query": "SELECT id, name FROM items"
				}
			}
		]
	}`

	setup(t, taskFile, taskJSON)
	defer teardown(t, taskFile)

	cmdAdd := exec.Command("../tooka", "add", taskFile)
	runCommand(t, cmdAdd)

	cmdRun := exec.Command("../tooka", "run", "sqlite-task")
	output, err := cmdRun.CombinedOutput()

	if err != nil {
		t.Fatalf("Failed to run task: %v", err)
	}

	if !strings.Contains(string(output), "Success") {
		t.Errorf("Expected SQL result to include 'apple', got: %s", string(output))
	}
}
