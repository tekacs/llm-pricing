# LLM Pricing

A CLI tool to visualize OpenRouter model pricing and calculate actual request costs in a clean, tabular format.

## Features

- 📊 **Tabular display** of model pricing per 1M tokens
- 🧮 **Cost calculation** for actual requests with input/output tokens
- 💾 **Cache pricing** support with TTL-based pricing (5min vs 1h)
- 🔍 **Filter models** by name or provider (e.g., `anthropic`, `sonnet`)
- 📝 **Verbose mode** showing all model details
- 🌐 **Live data** fetched from [OpenRouter API](https://openrouter.ai/api/v1/models) ([docs](https://openrouter.ai/docs/overview/models), [api reference](https://openrouter.ai/docs/api-reference/list-available-models))

## Quick Start

Calculate the cost of a request with 10,000 input tokens, 200 output tokens, and 9,500 cached tokens:

```bash
llm-pricing calc 10000 200 -c 9500 opus-4 gpt-4.1
```

```
Cost calculation: 10000 input + 200 output (9500 cached, 5m TTL)

Model                   | Input     | Output    | Cache Read | Cache Write | Total    
------------------------+-----------+-----------+------------+-------------+----------
anthropic/claude-opus-4 | $0.000000 | $0.015000 | $0.014250  | $0.009375   | $0.038625
openai/gpt-4.1          | $0.001000 | $0.001600 | $0.004750  | $0.000000   | $0.007350
openai/gpt-4.1-mini     | $0.000200 | $0.000320 | $0.000950  | $0.000000   | $0.001470
openai/gpt-4.1-nano     | $0.000050 | $0.000080 | $0.000237  | $0.000000   | $0.000367
```

## Installation

### From Releases

Download the latest binary for your platform from the [releases page](https://github.com/tekacs/llm-pricing/releases).

### From crates.io

```bash
cargo install llm-pricing
```

<details>
<summary>From Source</summary>

```bash
git clone https://github.com/tekacs/llm-pricing.git
cd llm-pricing
cargo install --path .
```
</details>

## Usage

### Calculate Request Costs

Calculate the actual cost of a request with specific token counts:

```bash
llm-pricing calc 10000 200 opus-4
```

```
Cost calculation: 10000 input + 200 output

Model                   | Input     | Output    | Total    
------------------------+-----------+-----------+----------
anthropic/claude-opus-4 | $0.150000 | $0.015000 | $0.165000
```

With cached tokens (uses 5-minute TTL by default):

```bash
llm-pricing calc 10000 200 -c 9500 opus-4
```

```
Cost calculation: 10000 input + 200 output (9500 cached, 5m TTL)

Model                   | Input     | Output    | Cache Read | Cache Write | Total    
------------------------+-----------+-----------+------------+-------------+----------
anthropic/claude-opus-4 | $0.000000 | $0.015000 | $0.014250  | $0.009375   | $0.038625
```

With 1-hour cache TTL (higher write costs):

```bash
llm-pricing calc 10000 200 -c 9500 --ttl 60 opus-4
```

#### Understanding Cache vs No-Cache Pricing

The `-c` flag indicates you're using caching rules, which affects pricing even when no tokens are cached:

**Without `-c` flag (no caching):**
```bash
llm-pricing calc 10000 200 opus-4
```
```
Cost calculation: 10000 input + 200 output

Model                   | Input     | Output    | Total    
------------------------+-----------+-----------+----------
anthropic/claude-opus-4 | $0.150000 | $0.015000 | $0.165000
```

**With `-c 0` flag (using caching, 0 cached tokens):**
```bash
llm-pricing calc 10000 200 -c 0 opus-4
```
```
Cost calculation: 10000 input + 200 output

Model                   | Input     | Output    | Cache Read | Cache Write | Total    
------------------------+-----------+-----------+------------+-------------+----------
anthropic/claude-opus-4 | $0.000000 | $0.015000 | $0.000000  | $0.187500   | $0.202500
```

When using caching (`-c` flag), all new tokens are written to cache at cache write prices (1.25x base price for 5-minute TTL), which replaces the regular input cost.

### List Models

#### Basic Usage

Show all models in a table format:

```bash
llm-pricing
```

```
Model                                     | Input | Output | Cache Read | Cache Write
------------------------------------------+-------+--------+------------+------------
anthropic/claude-opus-4                   | 15.00 | 75.00  | 1.50       | 18.75      
anthropic/claude-sonnet-4                 | 3.00  | 15.00  | 0.30       | 3.75       
google/gemini-2.5-pro                     | 1.25  | 10.00  | N/A        | N/A        
x-ai/grok-4                               | 3.00  | 15.00  | 0.75       | N/A        
openai/gpt-4o                             | 2.50  | 10.00  | N/A        | N/A        
...
```

### Filter by Provider

Show only Anthropic models:

```bash
llm-pricing anthropic
```

```
Model                                     | Input | Output | Cache Read | Cache Write
------------------------------------------+-------+--------+------------+------------
anthropic/claude-opus-4                   | 15.00 | 75.00  | 1.50       | 18.75      
anthropic/claude-sonnet-4                 | 3.00  | 15.00  | 0.30       | 3.75       
anthropic/claude-3.5-sonnet               | 3.00  | 15.00  | 0.30       | 3.75       
anthropic/claude-3.5-haiku                | 0.80  | 4.00   | 0.08       | 1.00       
anthropic/claude-3-opus                   | 15.00 | 75.00  | 1.50       | 18.75      
...
```

### Filter by Model Name

Show models containing "sonnet":

```bash
llm-pricing sonnet
```

```
Model                                     | Input | Output | Cache Read | Cache Write
------------------------------------------+-------+--------+------------+------------
anthropic/claude-sonnet-4                 | 3.00  | 15.00  | 0.30       | 3.75       
anthropic/claude-3.7-sonnet               | 3.00  | 15.00  | 0.30       | 3.75       
anthropic/claude-3.5-sonnet               | 3.00  | 15.00  | 0.30       | 3.75       
anthropic/claude-3-sonnet                 | 3.00  | 15.00  | 0.30       | 3.75       
```

### Verbose Output

Get detailed information about models with the `-v` flag:

```bash
llm-pricing opus-4 -v
```

```
=== ANTHROPIC ===

Model: anthropic/claude-opus-4
  Name: Anthropic: Claude Opus 4
  Description: Claude Opus 4 is benchmarked as the world's best coding model, at time of release, 
  bringing sustained performance on complex, long-running tasks and agent workflows. It sets new 
  benchmarks in software engineering, achieving leading results on SWE-bench (72.5%) and 
  Terminal-bench (43.2%).
  Pricing:
    Input: $15.00 per 1M tokens
    Output: $75.00 per 1M tokens
    Cache Read: $1.50 per 1M tokens
    Cache Write: $18.75 per 1M tokens
    Per Request: $0
    Image: $0.024
  Context Length: 200000 tokens
  Modality: text+image->text
  Tokenizer: Claude
  Max Completion Tokens: 32000
  Moderated: true
```

## Understanding the Output

### Table Columns

- **Model**: The model identifier used in API calls
- **Input**: Cost per 1M input tokens (USD)
- **Output**: Cost per 1M output tokens (USD)  
- **Cache Read**: Cost per 1M tokens read from cache (when available)
- **Cache Write**: Cost per 1M tokens written to cache (when available)

### Cache Pricing

Some providers (like Anthropic and xAI) offer caching to reduce costs on repeated content:

- **Cache Read**: Much cheaper than regular input tokens (typically 10x less)
- **Cache Write**: Slightly more expensive than input tokens (to build the cache)
- **N/A**: Model doesn't support caching

## CLI Options

### List Command (Default)

```bash
llm-pricing [OPTIONS] [FILTERS...]

Arguments:
  [FILTERS...]  Filter models by name (e.g., 'anthropic/', 'sonnet')

Options:
  -v, --verbose  Show verbose output with all model information
  -h, --help     Print help
```

### Calculate Command

```bash
llm-pricing calc [OPTIONS] <INPUT> <OUTPUT> [FILTERS...]

Arguments:
  <INPUT>       Number of input tokens
  <OUTPUT>      Number of output tokens
  [FILTERS...]  Filter models by name (e.g., 'anthropic/', 'sonnet')

Options:
  -c, --cached <CACHED>  Number of cached input tokens read from cache. Using this flag enables caching pricing rules.
  -t, --ttl <TTL>        Cache TTL in minutes (affects pricing) [default: 5]
  -h, --help             Print help
```

## Development

This project uses [just](https://github.com/casey/just) for task running:

```bash
# Show available tasks
just

# Build the project
just build

# Run with arguments
just run anthropic -v

# Format and lint
just fmt
just clippy
```

## License

MIT License - see [LICENSE](LICENSE) for details.
