# Discovering Apps

## List All Apps

```bash
infs app list
```

## Filter by Category

```bash
infs app list --category image
infs app list --category llm
infs app list --category video
infs app list --category audio
infs app list --category other
```

## Filter by Provider

```bash
infs app list --provider openrouter
infs app list --provider falai
infs app list --provider replicate
infs app list --provider wavespeed
```

## Combine Filters

Filters can be applied together (category and provider) using consecutive flags:

```bash
infs app list --category image --provider falai
```

## Paginate Results

Large catalogs are paginated.  Use `--page` and `--per-page` to navigate:

```bash
# First page (default: 20 per page)
infs app list

# Second page
infs app list --page 2

# 50 results per page
infs app list --per-page 50 --page 1
```

## Get App Details

```bash
infs app show openrouter/anthropic/claude-sonnet-4-5
infs app show falai/fal-ai/flux/dev
infs app show wavespeed/wavespeed-ai/flux-schnell
infs app show replicate/stability-ai/sdxl
```

## Machine-readable Output

```bash
# JSON list — useful for scripting
infs --json app list --category llm

# JSON for a specific app
infs --json app show openrouter/openai/gpt-4o
```

Example JSON response from `infs --json app list`:

```json
{
  "total": 42,
  "page": 1,
  "per_page": 20,
  "total_pages": 3,
  "apps": [
    {
      "full_id": "openrouter/openai/gpt-4o",
      "name": "GPT-4o",
      "category": "llm",
      "description": "OpenAI GPT-4o",
      "tags": ["llm", "openai"]
    }
  ]
}
```

## Provider Details

```bash
infs provider list
infs provider show openrouter
infs provider show falai
```
