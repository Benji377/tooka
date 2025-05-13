package task

import "time"

type Priority int

const (
	Low Priority = iota
	Medium
	Severe
)

func (p Priority) String() string {
	switch p {
	case Low:
		return "Low"
	case Medium:
		return "Medium"
	case Severe:
		return "Severe"
	default:
		return "Unknown"
	}
}

type Task struct {
	ID          int
	Title       string
	Description string
	DueDate     time.Time
	Completed   bool
	CreatedAt   time.Time
	Priority    Priority
}
