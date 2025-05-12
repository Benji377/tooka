package modules

import (
	"database/sql"
	"fmt"
)

type DBModule struct {
	DSN  string
	SQL  string
}

func NewDBModule(config map[string]any) (Module, error) {
	dsn, ok := config["dsn"].(string)
	sqlQuery, okSQL := config["sql"].(string)

	if !ok || dsn == "" || !okSQL || sqlQuery == "" {
		return nil, fmt.Errorf("missing or invalid 'dsn' (Data Source Name) or 'sql' query in database module config")
	}

	return &DBModule{
		DSN: dsn,
		SQL: sqlQuery,
	}, nil
}

func (m *DBModule) Run() string {
	// Open the database connection
	db, err := sql.Open("sqlite3", m.DSN) // Change this to your preferred database driver
	if err != nil {
		return fmt.Sprintf("Error: %s", err.Error())
	}
	defer db.Close()

	// Execute the SQL query
	rows, err := db.Query(m.SQL)
	if err != nil {
		return fmt.Sprintf("Error executing SQL query: %s", err.Error())
	}
	defer rows.Close()

	// Process query results (this example assumes a SELECT query returning columns 'id' and 'name')
	var result string
	for rows.Next() {
		var id int
		var name string
		if err := rows.Scan(&id, &name); err != nil {
			return fmt.Sprintf("Error scanning result: %s", err.Error())
		}
		result += fmt.Sprintf("ID: %d, Name: %s\n", id, name)
	}

	// Return any rows as a formatted string
	return result
}

func init() {
	RegisterModule(ModuleInfo{
		Name:        "db",
		Description: "Executes SQL queries against a database",
		ConfigHelp:  "Required: 'dsn' (string), 'sql' (string)",
		Constructor: NewDBModule,
	})
}
