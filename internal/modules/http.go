package modules

import (
	"bytes"
	"fmt"
	"io"
	"net/http"
	"time"
)

type HTTPModule struct {
	URL     string
	Method  string
	Headers map[string]string
	Payload string
	Timeout time.Duration
}

func NewHTTPModule(config map[string]any) (Module, error) {
	url, ok := config["url"].(string)
	if !ok || url == "" {
		return nil, fmt.Errorf("missing or invalid 'url'")
	}

	method := "GET"
	if m, ok := config["method"].(string); ok {
		method = m
	}

	headers := map[string]string{}
	if h, ok := config["headers"].(map[string]any); ok {
		for k, v := range h {
			if s, ok := v.(string); ok {
				headers[k] = s
			}
		}
	}

	payload := ""
	if p, ok := config["payload"].(string); ok {
		payload = p
	}

	timeout := 5 * time.Second
	if tStr, ok := config["timeout"].(string); ok {
		if t, err := time.ParseDuration(tStr); err == nil {
			timeout = t
		}
	}

	return &HTTPModule{
		URL:     url,
		Method:  method,
		Headers: headers,
		Payload: payload,
		Timeout: timeout,
	}, nil
}

func (m *HTTPModule) Run() string {
	client := &http.Client{Timeout: m.Timeout}
	var body io.Reader
	if m.Payload != "" {
		body = bytes.NewBuffer([]byte(m.Payload))
	}

	req, err := http.NewRequest(m.Method, m.URL, body)
	if err != nil {
		return fmt.Sprintf("Failed to create request: %v", err)
	}
	for k, v := range m.Headers {
		req.Header.Set(k, v)
	}

	resp, err := client.Do(req)
	if err != nil {
		return fmt.Sprintf("Request error: %v", err)
	}
	defer func() {
		if cerr := resp.Body.Close(); cerr != nil {
			fmt.Printf("Error closing response body: %v\n", cerr)
		}
	}()

	respBody, err := io.ReadAll(resp.Body)
	if err != nil {
		return fmt.Sprintf("Failed to read response: %v", err)
	}
	return fmt.Sprintf("Status: %s\nBody: %s", resp.Status, string(respBody))
}

func init() {
	RegisterModule(ModuleInfo{
		Name:        "http",
		Description: "Sends an HTTP request",
		ConfigHelp:  "Required: 'url'; Optional: 'method', 'headers', 'payload', 'timeout'",
		Constructor: NewHTTPModule,
	})
}
