# Markdown Content Negotiation

**`SWS`** provides an optional content negotiation feature that serves markdown files when clients explicitly request them via the `Accept: text/markdown` HTTP header.

This feature enables serving the raw markdown source of your documentation or content alongside the rendered HTML versions, allowing clients to choose their preferred format.

The HTTP methods supported are `GET` and `HEAD`.

This feature is disabled by default and can be controlled by the boolean `--accept-markdown` option or the equivalent [SERVER_ACCEPT_MARKDOWN](../configuration/environment-variables.md#server_accept_markdown) env.

## How it works

When a client sends a request with the `Accept: text/markdown` header, SWS will search for markdown variants in the following order:

1. `path.md` - Direct markdown file
2. `path.html.md` - Markdown source file
3. `path/index.html.md` - Directory index markdown source

If a markdown variant is found, it will be served with the `Content-Type: text/markdown; charset=utf-8` header. If no markdown variant exists, the request falls back to normal static file handling.

**Important**: The feature only activates for the explicit `Accept: text/markdown` header. Wildcard headers like `Accept: text/*` or `Accept: */*` will not trigger markdown content negotiation.

## Usage examples

### Basic usage

Start the server with markdown content negotiation enabled:

```sh
static-web-server --root ./public --accept-markdown
```

Request a markdown file:

```sh
curl -H "Accept: text/markdown" http://localhost:8080/article
# Returns: article.md if it exists
```

### With environment variable

```sh
export SERVER_ACCEPT_MARKDOWN=true
static-web-server --root ./public
```

### Configuration file

```toml
[general]
root = "./public"
accept-markdown = true
```

## Use cases

### LLM-friendly content with llms.txt

This feature is the perfect companion for the [llms.txt standard](https://llmstxt.org/), which recommends that websites provide clean markdown versions of their pages for Large Language Models (LLMs) to consume.

According to the llms.txt standard, pages should provide markdown versions at the same URL with `.md` appended (or `index.html.md` for URLs without file names). With SWS's markdown content negotiation, LLMs and AI agents can request these markdown versions using the `Accept: text/markdown` header:

```sh
# LLM requesting markdown version
curl -H "Accept: text/markdown" https://example.com/docs/api
# Returns: docs/api.html.md if it exists

# Directory index
curl -H "Accept: text/markdown" https://example.com/docs/
# Returns: docs/index.html.md if it exists
```

This approach is superior to traditional `.md` URL suffixes because:

- Uses standard HTTP content negotiation
- Same URL serves both HTML (browsers) and Markdown (LLMs)
- No need for separate URL structures or redirects
- Follows REST principles for resource representation

### Documentation sites

Serve both HTML and markdown versions of your documentation:

```
docs/
├── article.html       # Rendered version
└── article.html.md    # Markdown source
```

Browsers get the HTML version, while API clients or tools can request the markdown source:

```sh
# Browser request (default)
curl http://localhost:8080/article
# Returns: article.html

# Request markdown source
curl -H "Accept: text/markdown" http://localhost:8080/article
# Returns: article.html.md
```

### API responses

Allow your API clients to request content in markdown format:

```javascript
fetch('/documentation', {
  headers: { 'Accept': 'text/markdown' }
})
.then(response => response.text())
.then(markdown => console.log(markdown));
```

### Content management

Serve markdown files for editing tools while providing HTML for end users:

```sh
# Editor fetching source
curl -H "Accept: text/markdown" http://localhost:8080/blog/post
# Returns: blog/post.md

# Regular browser visit
curl http://localhost:8080/blog/post
# Returns: blog/post.html or blog/post/index.html
```
