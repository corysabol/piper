# Piper - Pipeline Automation with LLM Integration

Piper is a tool for executing pipelines in a repeatable manner, combining shell commands, Lua scripting, and LLM inference capabilities. It's designed to be a flexible and powerful automation tool, particularly useful for cybersecurity tasks like reconnaissance and penetration testing.

## Features

- Custom DSL (Domain Specific Language) for defining pipelines
- First-class LLM integration for intelligent processing
- Shell command execution
- Lua scripting support
- Variable interpolation
- HTTP requests
- Notification system
- Remote agent support

## Project Structure

- `piper`: Main CLI application
- `piper_dsl`: DSL parser implementation using pest
- `piper_runner`: Pipeline execution engine
- `piper_tasks`: Task implementations (cmd, http, llm, etc.)
- `piper_agent`: Remote agent implementation
- `notifications`: Notification system

## DSL Syntax

Piper uses a custom DSL for defining pipelines. Here's a basic example:

```
pipeline example {
    metadata {
        name: "Example Pipeline",
        author: "Piper Team",
        description: "A simple example pipeline",
        version: "1.0.0",
    }

    tasks {
        // Set a variable
        set_var {
            var: "greeting",
            val: "Hello, World!",
        }

        // Run a shell command
        cmd echo_greeting {
            cmd: "echo '#{greeting}'",
            description: "Echo the greeting",
        }
    }
}
```

### Task Types

- `cmd`: Execute shell commands
- `script`: Run Lua scripts
- `llm`: Perform LLM inference
- `http`: Make HTTP requests
- `notify`: Send notifications
- `set_var`: Set variables
- `lua`: Execute Lua code

## LLM Integration

Piper provides first-class LLM integration through the `llm` task type. Here's an example:

```
llm analyze {
    model: "mistralai/Mistral-7B-Instruct-v0.2",
    prompt: "Analyze the following data: #{data}",
    system: "You are a helpful assistant that analyzes data.",
    temperature: 0.3,
    max_tokens: 1024,
    output: "analysis_result",
}
```

### LLM Task Properties

- `model`: The LLM model to use (defaults to "mistralai/Mistral-7B-Instruct-v0.2")
- `prompt`: The prompt to send to the LLM
- `system`: Optional system prompt to set the context
- `temperature`: Optional temperature parameter (0.0 to 1.0)
- `max_tokens`: Optional maximum number of tokens to generate
- `output`: Optional variable name to store the LLM output

## Usage

### Installation

```bash
cargo install --path .
```

### Running a Pipeline

```bash
piper run -p pipelines/example.piper
```

### Running a Pipeline on a Remote Agent

```bash
piper run -p pipelines/example.piper -r -a http://agent-address:50051
```

### Starting an Agent

```bash
piper start-agent
```

## Example: Reconnaissance Pipeline

See `pipelines/recon.piper` for a complete example of a reconnaissance pipeline that:

1. Runs various reconnaissance tools (nmap, subfinder, gobuster)
2. Processes the results with LLM to identify security issues
3. Generates a comprehensive security report
4. Saves the report to a file
5. Sends a notification upon completion

## Basic CLI Usage
```
piper 0.1.0

USAGE:
    piper <SUBCOMMAND>

OPTIONS:
    -h, --help       Print help information
    -V, --version    Print version information

SUBCOMMANDS:
    agents         Manage remote agents
    help           Print this message or the help of the given subcommand(s)
    init           Initialize a project directory or agent with a config file
    run            Run a pipeline
    start-agent    Start in agent mode
    status         Check the status of a remote pipeline
```

## License

[MIT License](LICENSE)
