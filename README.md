# rust-llm-cli

A small command-line LLM client written in Rust, backed by the [OpenRouter](https://openrouter.ai) chat completions API.

## Setup

Create a `.env` file in the project root:

```
OPENROUTER_API_KEY=your_key_here
```

## Usage

```bash
cargo run -- ask "what is AI"
```

The reply is printed along with the finish reason, response id, and model.

### Commands

| Command | Description |
| --- | --- |
| `ask <prompt>` | Send a prompt to the model |
| `version` | Print the CLI version *(stub — still routed to the model)* |
| `models` | List available models *(stub — still routed to the model)* |

## Layout

- [src/main.rs](src/main.rs) — entry point, wires config → command → client
- [src/commands.rs](src/commands.rs) — clap CLI definition
- [src/config.rs](src/config.rs) — loads `OPENROUTER_API_KEY` from `.env`
- [src/client.rs](src/client.rs) — blocking HTTP call to OpenRouter
- [src/models.rs](src/models.rs) — response types
