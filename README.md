# LLM Pricing

A CLI tool to visualize OpenRouter model pricing in a clean, tabular format.

## Features

- 📊 **Tabular display** of model pricing per 1M tokens
- 🔍 **Filter models** by name or provider (e.g., `anthropic`, `sonnet`)
- 💰 **Cache pricing** support for models that offer it
- 📝 **Verbose mode** showing all model details
- 🌐 **Live data** fetched from OpenRouter API

## Installation

### From crates.io

```bash
cargo install llm-pricing
```

### From Releases

Download the latest binary for your platform from the [releases page](https://github.com/tekacs/llm-pricing/releases).

### From Source

```bash
git clone https://github.com/tekacs/llm-pricing.git
cd llm-pricing
cargo install --path .
```

## Usage

### Basic Usage

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

```bash
llm-pricing [OPTIONS] [FILTER]

Arguments:
  [FILTER]  Filter models by name (e.g., 'anthropic/', 'sonnet')

Options:
  -v, --verbose  Show verbose output with all model information
  -h, --help     Print help
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
