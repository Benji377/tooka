package modules

import (
	"database/sql"
	"fmt"
	"strings"
)

type SQLModule struct {
	DBPath string
	Query  string
}

func NewSQLModule(config map[string]any) (Module, error) {
	path, ok1 := config["db"].(string)
	query, ok2 := config["query"].(string)

	if !ok1 || !ok2 || path == "" || query == "" {
		return nil, fmt.Errorf("'db' (file path) and 'query' are required for SQL module")
	}

	return &SQLModule{
		DBPath: path,
		Query:  query,
	}, nil
}

func (m *SQLModule) Run() string {
	db, err := sql.Open("sqlite3", m.DBPath)
	if err != nil {
		return fmt.Sprintf("Failed to open DB: %v", err)
	}
	defer db.Close()

	rows, err := db.Query(m.Query)
	if err != nil {
		return fmt.Sprintf("Query failed: %v", err)
	}
	defer rows.Close()

	cols, _ := rows.Columns()
	var results []string
	for rows.Next() {
		data := make([]any, len(cols))
		ptrs := make([]any, len(cols))
		for i := range data {
			ptrs[i] = &data[i]
		}
		if err := rows.Scan(ptrs...); err != nil {
			continue
		}
		rowStr := ""
		for i, val := range data {
			rowStr += fmt.Sprintf("%s: %v  ", cols[i], val)
		}
		results = append(results, rowStr)
	}
	if len(results) == 0 {
		return "No rows returned."
	}
	return strings.Join(results, "\n")
}

func init() {
	RegisterModule(ModuleInfo{
		Name:        "sql",
		Description: "Executes a SQL query on a SQLite DB file",
		ConfigHelp:  "Required: 'db' (string path), 'query' (SQL string)",
		Constructor: NewSQLModule,
	})
}
