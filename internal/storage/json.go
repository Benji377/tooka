package storage

import (
    "encoding/json"
    "os"
    "path/filepath"

    "github.com/Benji377/tooka/internal/task"
)

var taskFile = filepath.Join(os.TempDir(), "tooka_tasks.json")

func SaveTasks(tasks []task.Task) error {
    data, err := json.MarshalIndent(tasks, "", "  ")
    if err != nil {
        return err
    }
    return os.WriteFile(taskFile, data, 0644)
}

func LoadTasks() ([]task.Task, error) {
    var tasks []task.Task
    data, err := os.ReadFile(taskFile)
    if err != nil {
        if os.IsNotExist(err) {
            return tasks, nil
        }
        return nil, err
    }
    err = json.Unmarshal(data, &tasks)
    return tasks, err
}
