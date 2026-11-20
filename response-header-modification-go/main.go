package main

import (
	"io"
	"net/http"

	spinhttp "github.com/spinframework/spin-go-sdk/v3/http"
	"github.com/spinframework/spin-go-sdk/v3/variables"
)

func init() {
	spinhttp.Handle(func(w http.ResponseWriter, r *http.Request) {
		if r.Method != http.MethodGet {
			http.Error(w, "method not allowed", http.StatusMethodNotAllowed)
			return
		}

		// Read the origin from the Spin variable "origin" (see spin.toml)
		origin, err := variables.Get("origin")
		if err != nil || origin == "" {
			http.Error(w, "origin variable not set", http.StatusInternalServerError)
			return
		}

		// Build the origin URL: origin + request path (query string follows below)
		req, err := http.NewRequest(http.MethodGet, origin+r.URL.Path, nil)
		if err != nil {
			http.Error(w, err.Error(), http.StatusInternalServerError)
			return
		}

		// Append the query string. spin-go-sdk v3.0.0 only sends req.URL.Path
		// to the host and drops req.URL.RawQuery, so we add it to the path ourselves.
		if r.URL.RawQuery != "" {
			req.URL.Path += "?" + r.URL.RawQuery
		}

		// Forward the accept header of the incoming request (default: */*)
		accept := r.Header.Get("accept")
		if accept == "" {
			accept = "*/*"
		}
		req.Header.Set("accept", accept)

		// Send the outbound request using the Spin SDK
		res, err := spinhttp.Send(req)
		if err != nil {
			http.Error(w, "upstream request failed", http.StatusBadGateway)
			return
		}
		defer res.Body.Close()

		// Respond with the origin's status and body, but only pass through
		// the content-type header and add our own x-intercepted-value header
		w.Header().Set("content-type", res.Header.Get("content-type"))
		w.Header().Set("x-intercepted-value", "foobar")
		w.WriteHeader(res.StatusCode)
		io.Copy(w, res.Body)
	})
}

// main function must be included for the compiler but is not executed.
func main() {}
