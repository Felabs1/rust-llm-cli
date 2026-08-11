# rust-llm-cli

A small command-line LLM client written in Rust, backed by the [OpenRouter](https://openrouter.ai) chat completions API.

## Setup

Create a `.env` file in the project root:

```
OPENROUTER_API_KEY=your_key_here
```

## Usage

```bash
cargo run -- ask
```

`ask` starts an interactive chat loop. It prompts with `You: `, prints the
model's reply as `AI: ...`, and keeps going until you type `exit`.

```
You: what is AI
AI: ...
History: 2 messages, ~12 tokens
You: exit
```

The full conversation is sent to the model on each turn, so it has context from
earlier messages. After every turn the CLI prints how many messages the history
holds and a rough token estimate.

### History truncation

Tokens are estimated as `content.len() / 4` per message. When the history
exceeds `MAX_TOKENS` (50, in [src/main.rs](src/main.rs)), the oldest
user/assistant pair is dropped, repeating until the history fits or only one
turn remains.

### Commands

| Command | Description |
| --- | --- |
| `ask` | Start the interactive chat loop |

## Layout

- [src/main.rs](src/main.rs) — entry point, chat loop, history truncation
- [src/commands.rs](src/commands.rs) — clap CLI definition and stdin prompt reader
- [src/config.rs](src/config.rs) — loads `OPENROUTER_API_KEY` from `.env`
- [src/client.rs](src/client.rs) — blocking HTTP call to OpenRouter, sends the message history
- [src/models.rs](src/models.rs) — request/response types
