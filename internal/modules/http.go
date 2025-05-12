package modules

import (
	"fmt"
	"io/ioutil"
	"net/http"
	"strings"
)

type HTTPModule struct {
	URL         string
	Method      string
	Headers     map[string]string
	Body        string
	ReturnType  string
}

func NewHTTPModule(config map[string]any) (Module, error) {
	url, ok := config["url"].(string)
	method, okMethod := config["method"].(string)
	headers, okHeaders := config["headers"].(map[string]string)
	body, okBody := config["body"].(string)
	returnType, okReturn := config["return_type"].(string)

	if !ok || url == "" {
		return nil, fmt.Errorf("missing or invalid 'url' in HTTP module config")
	}
	if !okMethod {
		method = "GET" // Default to GET method if not specified
	}
	if !okHeaders {
		headers = make(map[string]string) // Default to empty headers
	}
	if !okBody {
		body = "" // Default to empty body
	}
	if !okReturn {
		returnType = "text" // Default return type to text
	}

	return &HTTPModule{
		URL:        url,
		Method:     method,
		Headers:    headers,
		Body:       body,
		ReturnType: returnType,
	}, nil
}

func (m *HTTPModule) Run() string {
	// Create a new HTTP request
	req, err := http.NewRequest(m.Method, m.URL, strings.NewReader(m.Body))
	if err != nil {
		return fmt.Sprintf("Error: %s", err.Error())
	}

	// Set headers if any
	for key, value := range m.Headers {
		req.Header.Set(key, value)
	}

	// Perform the HTTP request
	client := &http.Client{}
	resp, err := client.Do(req)
	if err != nil {
		return fmt.Sprintf("Error: %s", err.Error())
	}
	defer resp.Body.Close()

	// Read the response body
	body, err := ioutil.ReadAll(resp.Body)
	if err != nil {
		return fmt.Sprintf("Error reading response body: %s", err.Error())
	}

	// Depending on the return type, process accordingly
	if m.ReturnType == "json" {
		// In a real-world scenario, you could attempt to unmarshal JSON here if necessary
		return fmt.Sprintf("JSON Response: %s", string(body))
	}

	return string(body) // Return the body as text
}

func init() {
	RegisterModule(ModuleInfo{
		Name:        "http",
		Description: "Performs an HTTP request (GET, POST, etc.)",
		ConfigHelp:  "Required: 'url' (string), 'method' (string), 'headers' (map[string]string), 'body' (string), 'return_type' (string)",
		Constructor: NewHTTPModule,
	})
}
