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

### LLM Task Properties

- `model`: The LLM model to use (defaults to "mistralai/Mistral-7B-Instruct-v0.2")
- `prompt`: The prompt to send to the LLM
- `system`: Optional system prompt to set the context
- `temperature`: Optional temperature parameter (0.0 to 1.0)
- `max_tokens`: Optional maximum number of tokens to generate
- `output`: Optional variable name to store the LLM output

### A better example
Program structure
```
pipeline <name>(<args>) {
  // 1. Constants (immutable, required)
  CONSTANT = value
  
  // 2. Task Definitions (named, required)
  task_name = builtin_task(...)
  
  // 3. Flow Sequence (required, last section)
  task1 > task2 > [task3, task4] > (cond ? task5 : task6)
}
```

### Key DSL Features

The example above demonstrates several key features of the Piper DSL:

1. **Named Data Literals**
   - Immutable variables: `TARGET_LIST = targets`
   - Complex data structures: `SCAN_CONFIG = { basic: {...}, thorough: {...} }`
   - Multiline strings: `SYSTEM_PROMPT = """..."""`

2. **Function-Call Style Tasks**
   - Tasks defined as function calls: `cmd("command", output="var")`
   - Named arguments for clarity: `command="...", output="..."`
   - Tasks assigned to variables: `task_name = task_type(...)`

3. **Explicit Flow Control**
   - Sequential execution with `>` operator
   - Parallel execution with `[task1, task2]` syntax
   - Conditional execution: `(condition ? task : null)`

4. **LLM Task Integration**
   - Separate `tasks` argument for LLM-executable tasks
   - LLM can choose which tasks to run based on analysis
   - Clean separation between prompt and available tools

5. **Variable Interpolation**
   - Simple interpolation: `#{variable}`
   - Object property access: `#{SCAN_CONFIG[scan_type].ports}`
   - Fallback values: `#{variable || "default"}`

6. **Pipeline Parameters**
   - Parameterized pipelines: `pipeline name(param="default")`
   - Parameters accessible throughout the pipeline

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
