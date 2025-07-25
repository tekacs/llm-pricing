use clap::Parser;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Parser, Debug)]
#[command(name = "llm-pricing")]
#[command(about = "A CLI tool to visualize OpenRouter model pricing")]
struct Args {
    #[command(subcommand)]
    command: Option<Commands>,

    // For backward compatibility when no subcommand is used
    /// Filter models by name (e.g., 'anthropic/', 'sonnet')
    #[arg(global = true)]
    filters: Vec<String>,

    /// Show verbose output with all model information
    #[arg(short, long, global = true)]
    verbose: bool,
}

#[derive(Parser, Debug)]
enum Commands {
    /// List models with pricing (default)
    List {
        /// Filter models by name (e.g., 'anthropic/', 'sonnet')
        filters: Vec<String>,
        /// Show verbose output with all model information
        #[arg(short, long)]
        verbose: bool,
    },
    /// Calculate cost for a specific request
    Calc {
        /// Number of input tokens
        input: u64,
        /// Number of output tokens
        output: u64,
        /// Filter models by name (e.g., 'anthropic/', 'sonnet')
        filters: Vec<String>,
        /// Number of cached input tokens (default: 0)
        #[arg(short, long, default_value = "0")]
        cached: u64,
        /// Cache TTL in minutes (affects pricing for some models, default: 5)
        #[arg(short, long, default_value = "5")]
        ttl: u64,
    },
}

#[derive(Debug, Deserialize, Serialize)]
struct Model {
    id: String,
    #[serde(default)]
    canonical_slug: Option<String>,
    #[serde(default)]
    hugging_face_id: Option<String>,
    name: Option<String>,
    #[serde(default)]
    created: Option<u64>,
    description: Option<String>,
    pricing: Pricing,
    context_length: Option<u64>,
    architecture: Option<Architecture>,
    top_provider: Option<TopProvider>,
    per_request_limits: Option<PerRequestLimits>,
    #[serde(default)]
    supported_parameters: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Pricing {
    prompt: String,
    completion: String,
    #[serde(default)]
    request: Option<String>,
    #[serde(default)]
    image: Option<String>,
    #[serde(default)]
    input_cache_read: Option<String>,
    #[serde(default)]
    input_cache_write: Option<String>,
    #[serde(default)]
    web_search: Option<String>,
    #[serde(default)]
    internal_reasoning: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Architecture {
    modality: Option<String>,
    #[serde(default)]
    input_modalities: Option<Vec<String>>,
    #[serde(default)]
    output_modalities: Option<Vec<String>>,
    tokenizer: Option<String>,
    instruct_type: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct TopProvider {
    #[serde(default)]
    context_length: Option<u64>,
    max_completion_tokens: Option<u64>,
    is_moderated: Option<bool>,
}

#[derive(Debug, Deserialize, Serialize)]
struct PerRequestLimits {
    #[serde(default)]
    prompt_tokens: Option<String>,
    #[serde(default)]
    completion_tokens: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct ApiResponse {
    data: Vec<Model>,
}

async fn fetch_models() -> anyhow::Result<Vec<Model>> {
    let client = reqwest::Client::new();
    let response = client
        .get("https://openrouter.ai/api/v1/models")
        .send()
        .await?;

    let api_response: ApiResponse = response.json().await?;
    Ok(api_response.data)
}

fn group_models_by_provider(models: Vec<Model>) -> HashMap<String, Vec<Model>> {
    let mut grouped = HashMap::new();

    for model in models {
        let provider = model.id.split('/').next().unwrap_or("unknown").to_string();
        grouped.entry(provider).or_insert_with(Vec::new).push(model);
    }

    grouped
}

fn filter_models(
    grouped: HashMap<String, Vec<Model>>,
    filters: Vec<String>,
) -> HashMap<String, Vec<Model>> {
    if filters.is_empty() {
        return grouped;
    }

    let filter_lowers: Vec<String> = filters.iter().map(|f| f.to_lowercase()).collect();

    let mut filtered = HashMap::new();
    for (provider, models) in grouped {
        let filtered_models: Vec<Model> = models
            .into_iter()
            .filter(|model| {
                filter_lowers.iter().any(|filter_lower| {
                    model.id.to_lowercase().contains(filter_lower)
                        || model
                            .name
                            .as_ref()
                            .is_some_and(|name| name.to_lowercase().contains(filter_lower))
                })
            })
            .collect();

        if !filtered_models.is_empty() {
            filtered.insert(provider, filtered_models);
        }
    }
    filtered
}

fn format_price_per_million(price_str: &str) -> String {
    if let Ok(price) = price_str.parse::<f64>() {
        format!("{:.2}", price * 1_000_000.0)
    } else {
        "N/A".to_string()
    }
}

fn parse_price(price_str: &str) -> anyhow::Result<f64> {
    price_str
        .parse::<f64>()
        .map_err(|e| anyhow::anyhow!("Invalid price format: {}", e))
}

struct TableRow {
    model: String,
    input: String,
    output: String,
    cache_read: String,
    cache_write: String,
}

fn print_default_format(grouped: &HashMap<String, Vec<Model>>) {
    let mut rows = Vec::new();

    for models in grouped.values() {
        for model in models {
            let input_price = format_price_per_million(&model.pricing.prompt);
            let output_price = format_price_per_million(&model.pricing.completion);

            let cache_read = model
                .pricing
                .input_cache_read
                .as_ref()
                .map(|p| format_price_per_million(p))
                .unwrap_or_else(|| "N/A".to_string());
            let cache_write = model
                .pricing
                .input_cache_write
                .as_ref()
                .map(|p| format_price_per_million(p))
                .unwrap_or_else(|| "N/A".to_string());

            rows.push(TableRow {
                model: model.id.clone(),
                input: input_price,
                output: output_price,
                cache_read,
                cache_write,
            });
        }
    }

    if rows.is_empty() {
        return;
    }

    // Calculate column widths
    let max_model_width = rows.iter().map(|r| r.model.len()).max().unwrap_or(0).max(5);
    let max_input_width = rows.iter().map(|r| r.input.len()).max().unwrap_or(0).max(5);
    let max_output_width = rows
        .iter()
        .map(|r| r.output.len())
        .max()
        .unwrap_or(0)
        .max(6);
    let max_cache_read_width = rows
        .iter()
        .map(|r| r.cache_read.len())
        .max()
        .unwrap_or(0)
        .max(10);
    let max_cache_write_width = rows
        .iter()
        .map(|r| r.cache_write.len())
        .max()
        .unwrap_or(0)
        .max(11);

    // Print header
    println!(
        "{:<width_model$} | {:<width_input$} | {:<width_output$} | {:<width_read$} | {:<width_write$}",
        "Model",
        "Input",
        "Output",
        "Cache Read",
        "Cache Write",
        width_model = max_model_width,
        width_input = max_input_width,
        width_output = max_output_width,
        width_read = max_cache_read_width,
        width_write = max_cache_write_width,
    );

    // Print separator
    println!(
        "{:-<width_model$}-+-{:-<width_input$}-+-{:-<width_output$}-+-{:-<width_read$}-+-{:-<width_write$}",
        "",
        "",
        "",
        "",
        "",
        width_model = max_model_width,
        width_input = max_input_width,
        width_output = max_output_width,
        width_read = max_cache_read_width,
        width_write = max_cache_write_width,
    );

    // Print rows
    for row in rows {
        println!(
            "{:<width_model$} | {:<width_input$} | {:<width_output$} | {:<width_read$} | {:<width_write$}",
            row.model,
            row.input,
            row.output,
            row.cache_read,
            row.cache_write,
            width_model = max_model_width,
            width_input = max_input_width,
            width_output = max_output_width,
            width_read = max_cache_read_width,
            width_write = max_cache_write_width,
        );
    }
}

fn print_verbose_format(grouped: &HashMap<String, Vec<Model>>) {
    for (provider, models) in grouped {
        println!("\n=== {} ===", provider.to_uppercase());

        for model in models {
            println!("\nModel: {}", model.id);

            if let Some(name) = &model.name {
                println!("  Name: {name}");
            }

            if let Some(description) = &model.description {
                println!("  Description: {description}");
            }

            println!("  Pricing:");
            println!(
                "    Input: ${} per 1M tokens",
                format_price_per_million(&model.pricing.prompt)
            );
            println!(
                "    Output: ${} per 1M tokens",
                format_price_per_million(&model.pricing.completion)
            );

            if let Some(cache_read) = &model.pricing.input_cache_read {
                println!(
                    "    Cache Read: ${} per 1M tokens",
                    format_price_per_million(cache_read)
                );
            }
            if let Some(cache_write) = &model.pricing.input_cache_write {
                println!(
                    "    Cache Write: ${} per 1M tokens",
                    format_price_per_million(cache_write)
                );
            }

            if let Some(request_price) = &model.pricing.request {
                println!("    Per Request: ${request_price}");
            }

            if let Some(image_price) = &model.pricing.image {
                println!("    Image: ${image_price}");
            }

            if let Some(context_length) = model.context_length {
                println!("  Context Length: {context_length} tokens");
            }

            if let Some(arch) = &model.architecture {
                if let Some(modality) = &arch.modality {
                    println!("  Modality: {modality}");
                }
                if let Some(tokenizer) = &arch.tokenizer {
                    println!("  Tokenizer: {tokenizer}");
                }
                if let Some(instruct_type) = &arch.instruct_type {
                    println!("  Instruct Type: {instruct_type}");
                }
            }

            if let Some(top_provider) = &model.top_provider {
                if let Some(max_completion) = top_provider.max_completion_tokens {
                    println!("  Max Completion Tokens: {max_completion}");
                }
                if let Some(is_moderated) = top_provider.is_moderated {
                    println!("  Moderated: {is_moderated}");
                }
            }
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let models = fetch_models().await?;

    match args.command {
        Some(Commands::List { filters, verbose }) => {
            let grouped = group_models_by_provider(models);
            let filtered = filter_models(grouped, filters);

            if verbose {
                print_verbose_format(&filtered);
            } else {
                print_default_format(&filtered);
            }
        }
        None => {
            // Default to list command for backward compatibility
            let grouped = group_models_by_provider(models);
            let filtered = filter_models(grouped, args.filters);

            if args.verbose {
                print_verbose_format(&filtered);
            } else {
                print_default_format(&filtered);
            }
        }
        Some(Commands::Calc {
            filters,
            input,
            output,
            cached,
            ttl,
        }) => {
            let grouped = group_models_by_provider(models);
            let filtered = filter_models(grouped, filters);

            struct CalcRow {
                model: String,
                input_cost: f64,
                output_cost: f64,
                cache_read_cost: f64,
                cache_write_cost: f64,
                total_cost: f64,
            }

            let mut calc_rows = Vec::new();

            for (_, models_in_provider) in filtered {
                for model in models_in_provider {
                    let input_price = parse_price(&model.pricing.prompt)?;
                    let output_price = parse_price(&model.pricing.completion)?;
                    let non_cached_input = input.saturating_sub(cached);

                    let input_cost = (non_cached_input as f64) * input_price;
                    let output_cost = (output as f64) * output_price;

                    let mut cache_read_cost = 0.0;
                    let mut cache_write_cost = 0.0;

                    if cached > 0 {
                        if let Some(cache_read_price_str) = &model.pricing.input_cache_read {
                            let cache_read_price = parse_price(cache_read_price_str)?;
                            cache_read_cost = (cached as f64) * cache_read_price;
                        } else {
                            cache_read_cost = (cached as f64) * input_price;
                        }

                        if model.pricing.input_cache_write.is_some() {
                            let actual_write_price = match ttl {
                                5 => input_price * 1.25, // 5-minute TTL is 1.25x base price
                                60 => input_price * 2.0,  // 1-hour TTL is 2x base price
                                _ => unimplemented!("TTL must be exactly 5 or 60 minutes"),
                            };
                            cache_write_cost = (cached as f64) * actual_write_price;
                        }
                    }

                    let total_cost = input_cost + output_cost + cache_read_cost + cache_write_cost;

                    calc_rows.push(CalcRow {
                        model: model.id.clone(),
                        input_cost,
                        output_cost,
                        cache_read_cost,
                        cache_write_cost,
                        total_cost,
                    });
                }
            }

            if calc_rows.is_empty() {
                eprintln!("No models found matching the filter");
                eprintln!("Use 'llm-pricing list' to see available models");
                std::process::exit(1);
            }

            // Calculate column widths
            let max_model_width = calc_rows
                .iter()
                .map(|r| r.model.len())
                .max()
                .unwrap_or(0)
                .max(5);

            // Calculate the actual width needed for cost columns
            let format_cost = |cost: f64| format!("${:.6}", cost);
            let max_input_width = calc_rows
                .iter()
                .map(|r| format_cost(r.input_cost).len())
                .max()
                .unwrap_or(0)
                .max(5);
            let max_output_width = calc_rows
                .iter()
                .map(|r| format_cost(r.output_cost).len())
                .max()
                .unwrap_or(0)
                .max(6);
            let max_cache_read_width = calc_rows
                .iter()
                .map(|r| format_cost(r.cache_read_cost).len())
                .max()
                .unwrap_or(0)
                .max(10);
            let max_cache_write_width = calc_rows
                .iter()
                .map(|r| format_cost(r.cache_write_cost).len())
                .max()
                .unwrap_or(0)
                .max(11);
            let max_total_width = calc_rows
                .iter()
                .map(|r| format_cost(r.total_cost).len())
                .max()
                .unwrap_or(0)
                .max(5);

            // Print header with request details
            let cache_desc = if cached > 0 {
                let ttl_desc = match ttl {
                    5 => "5m",
                    60 => "1h",
                    _ => unimplemented!("TTL must be exactly 5 or 60 minutes"),
                };
                format!(" ({} cached, {} TTL)", cached, ttl_desc)
            } else {
                String::new()
            };

            println!(
                "Cost calculation: {} input + {} output{}",
                input, output, cache_desc
            );
            println!();

            if cached > 0 {
                println!("{:<width_model$} | {:<width_input$} | {:<width_output$} | {:<width_read$} | {:<width_write$} | {:<width_total$}",
                    "Model", "Input", "Output", "Cache Read", "Cache Write", "Total",
                    width_model = max_model_width,
                    width_input = max_input_width,
                    width_output = max_output_width,
                    width_read = max_cache_read_width,
                    width_write = max_cache_write_width,
                    width_total = max_total_width);
                println!("{:-<width_model$}-+-{:-<width_input$}-+-{:-<width_output$}-+-{:-<width_read$}-+-{:-<width_write$}-+-{:-<width_total$}",
                    "", "", "", "", "", "",
                    width_model = max_model_width,
                    width_input = max_input_width,
                    width_output = max_output_width,
                    width_read = max_cache_read_width,
                    width_write = max_cache_write_width,
                    width_total = max_total_width);

                for row in calc_rows {
                    println!("{:<width_model$} | {:<width_input$} | {:<width_output$} | {:<width_read$} | {:<width_write$} | {:<width_total$}",
                        row.model,
                        format_cost(row.input_cost),
                        format_cost(row.output_cost),
                        format_cost(row.cache_read_cost),
                        format_cost(row.cache_write_cost),
                        format_cost(row.total_cost),
                        width_model = max_model_width,
                        width_input = max_input_width,
                        width_output = max_output_width,
                        width_read = max_cache_read_width,
                        width_write = max_cache_write_width,
                        width_total = max_total_width);
                }
            } else {
                println!(
                    "{:<width_model$} | {:<width_input$} | {:<width_output$} | {:<width_total$}",
                    "Model",
                    "Input",
                    "Output",
                    "Total",
                    width_model = max_model_width,
                    width_input = max_input_width,
                    width_output = max_output_width,
                    width_total = max_total_width
                );
                println!("{:-<width_model$}-+-{:-<width_input$}-+-{:-<width_output$}-+-{:-<width_total$}",
                    "", "", "", "",
                    width_model = max_model_width,
                    width_input = max_input_width,
                    width_output = max_output_width,
                    width_total = max_total_width);

                for row in calc_rows {
                    println!("{:<width_model$} | {:<width_input$} | {:<width_output$} | {:<width_total$}",
                        row.model,
                        format_cost(row.input_cost),
                        format_cost(row.output_cost),
                        format_cost(row.total_cost),
                        width_model = max_model_width,
                        width_input = max_input_width,
                        width_output = max_output_width,
                        width_total = max_total_width);
                }
            }
        }
    }

    Ok(())
}
