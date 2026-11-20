package main

import (
	"io"
	"net/http"
	"regexp"
	"strings"

	spinhttp "github.com/spinframework/spin-go-sdk/v3/http"
)

const swapiPeople = "https://swapi.dev/api/people/"

// digitsOnly matches ids like "1", "4" or "42"
var digitsOnly = regexp.MustCompile(`^[0-9]+$`)

func init() {
	spinhttp.Handle(func(w http.ResponseWriter, r *http.Request) {
		switch {
		// GET / returns Luke Skywalker
		case r.Method == http.MethodGet && r.URL.Path == "/":
			fetchPerson(w, "1")

		// GET /people/<id> returns the person with the given id
		case r.Method == http.MethodGet && strings.HasPrefix(r.URL.Path, "/people/"):
			id := strings.TrimPrefix(r.URL.Path, "/people/")
			if !digitsOnly.MatchString(id) {
				writeJSON(w, http.StatusBadRequest, `{"error":"id must be a number"}`)
				return
			}
			fetchPerson(w, id)

		default:
			writeJSON(w, http.StatusNotFound, `{"error":"not found"}`)
		}
	})
}

// fetchPerson loads a person from SWAPI and returns the JSON body unchanged
func fetchPerson(w http.ResponseWriter, id string) {
	// SWAPI expects a trailing slash, otherwise it redirects
	res, err := spinhttp.Get(swapiPeople + id + "/")
	if err != nil {
		writeJSON(w, http.StatusBadGateway, `{"error":"upstream request failed"}`)
		return
	}
	defer res.Body.Close()

	w.Header().Set("content-type", "application/json")
	w.WriteHeader(res.StatusCode)
	io.Copy(w, res.Body)
}

// writeJSON sends a JSON body with the given status code
func writeJSON(w http.ResponseWriter, status int, body string) {
	w.Header().Set("content-type", "application/json")
	w.WriteHeader(status)
	io.WriteString(w, body)
}

// main function must be included for the compiler but is not executed.
func main() {}
