command "web-search" {
  description = "Search the web via DuckDuckGo (no API key required)"
  mode        = "read"

  param "query" {
    type        = "string"
    required    = true
    description = "Search query"
  }

  operation {
    protocol = "http"
    method   = "GET"
    url      = "https://html.duckduckgo.com/html/"
    headers = {
      "User-Agent" = "kb-mcp-researcher/0.1"
    }
    query = {
      q = "{{ args.query }}"
    }
  }
}
