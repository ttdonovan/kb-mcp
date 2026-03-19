command "web-fetch" {
  description = "Fetch a web page and return its content"
  mode        = "read"

  param "url" {
    type     = "string"
    required = true
    description = "URL to fetch"
  }

  operation {
    protocol = "http"
    method   = "GET"
    url      = "{{ args.url }}"
    headers = {
      "User-Agent" = "kb-mcp-researcher/0.1"
      "Accept"     = "text/html,application/xhtml+xml,text/plain"
    }
  }
}
