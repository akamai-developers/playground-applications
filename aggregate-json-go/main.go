package main

import (
	"encoding/json"
	"fmt"
	"net/http"
	"sync"

	spinhttp "github.com/spinframework/spin-go-sdk/v3/http"
)

const (
	personURL = "https://swapi.dev/api/people/1/"
	planetURL = "https://swapi.dev/api/planets/1/"
)

// Both SWAPI resources have a "name" field, which is all we need
type named struct {
	Name string `json:"name"`
}

func init() {
	spinhttp.Handle(func(w http.ResponseWriter, r *http.Request) {
		if r.Method != http.MethodGet || r.URL.Path != "/" {
			writeJSON(w, http.StatusNotFound, map[string]string{"error": "not found"})
			return
		}

		// Fetch person and planet concurrently, one goroutine per request.
		// spin-go-sdk v3 uses async WASI HTTP (0.3), so while one goroutine
		// waits for its response, the other one can send its request.
		var person, planet named
		var personErr, planetErr error
		var wg sync.WaitGroup
		wg.Add(2)
		go func() { defer wg.Done(); personErr = fetchJSON(personURL, &person) }()
		go func() { defer wg.Done(); planetErr = fetchJSON(planetURL, &planet) }()
		wg.Wait()

		if personErr != nil || planetErr != nil {
			writeJSON(w, http.StatusBadGateway, map[string]string{"error": "upstream request failed"})
			return
		}

		writeJSON(w, http.StatusOK, map[string]string{
			"person": person.Name,
			"planet": planet.Name,
		})
	})
}

// fetchJSON sends a GET request using the Spin SDK and decodes the JSON body into v
func fetchJSON(url string, v any) error {
	res, err := spinhttp.Get(url)
	if err != nil {
		return err
	}
	defer res.Body.Close()

	if res.StatusCode < 200 || res.StatusCode > 299 {
		return fmt.Errorf("unexpected status %d from %s", res.StatusCode, url)
	}
	return json.NewDecoder(res.Body).Decode(v)
}

// writeJSON encodes v as JSON and sends it with the given status code
func writeJSON(w http.ResponseWriter, status int, v any) {
	w.Header().Set("content-type", "application/json")
	w.WriteHeader(status)
	json.NewEncoder(w).Encode(v)
}

// main function must be included for the compiler but is not executed.
func main() {}
