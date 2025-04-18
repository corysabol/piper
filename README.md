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

```
// Usage: piper recon targets=foo.txt scan_type=deep
pipeline recon(targets="targets.txt", scan_type="basic") {
  // Named data literals
  TARGET_LIST = targets
  OUTPUT_DIR = "./recon_results"
  SCAN_CONFIG = {
    basic: { ports: "top-1000", timing: "T3" },
    thorough: { ports: "1-65535", timing: "T4" }
  }
  
  // System prompt for the LLM
  SYSTEM_PROMPT = """
    You are a cybersecurity expert analyzing reconnaissance data.
    Identify potential vulnerabilities, misconfigurations, and security issues.
    If you find concerning issues, use the provided tools to investigate further.
  """
  
  // Define tasks as named variables
  // Read target list
  read_targets = cmd("cat #{TARGET_LIST}", output="host_list")
  
  // Run nmap scan
  nmap_scan = cmd(
    command="nmap -sV -p #{SCAN_CONFIG[scan_type].ports} -#{SCAN_CONFIG[scan_type].timing} -oN #{OUTPUT_DIR}/nmap.txt -iL #{TARGET_LIST}",
    description="Scanning network services",
    output="nmap_results"
  )
  
  // Run EyeWitness for web screenshots
  eyewitness = cmd(
    command="eyewitness --web -f #{TARGET_LIST} -d #{OUTPUT_DIR}/screenshots --no-prompt",
    description="Capturing web screenshots",
    output="screenshot_results"
  )
  
  // Parse nmap results to structured format
  parse_nmap = cmd(
    command="python parse_nmap.py --input #{OUTPUT_DIR}/nmap.txt --output #{OUTPUT_DIR}/nmap.json",
    output="parsed_nmap"
  )
  
  // Generate HTML report from screenshots
  process_screenshots = cmd(
    command="python process_screenshots.py --dir #{OUTPUT_DIR}/screenshots --output #{OUTPUT_DIR}/web_report.html",
    output="web_report"
  )
  
  // Define investigation tasks that the LLM might choose to run
  deep_scan = cmd(
    command="nmap -sV -p- -A -T4 #{host} -oN #{OUTPUT_DIR}/deep_scan_#{host}.txt",
    output="deep_scan_results"
  )
  
  vuln_scan = cmd(
    command="nmap --script vuln #{host} -oN #{OUTPUT_DIR}/vuln_scan_#{host}.txt",
    output="vuln_scan_results"
  )
  
  dir_brute = cmd(
    command="gobuster dir -u http://#{host} -w /usr/share/wordlists/dirb/common.txt -o #{OUTPUT_DIR}/dirs_#{host}.txt",
    output="dir_brute_results"
  )
  
  cve_lookup = http(
    url="https://cve.circl.lu/api/cve/#{cve_id}",
    method="GET",
    output="cve_details"
  )
  
  whois_lookup = cmd(
    command="whois #{domain} > #{OUTPUT_DIR}/whois_#{domain}.txt",
    output="whois_results"
  )
  
  // LLM analysis of reconnaissance data with tasks as an argument
  analyze_recon = llm(
    model="mistralai/Mistral-7B-Instruct-v0.2",
    system=SYSTEM_PROMPT,
    prompt="""
      I've performed reconnaissance on the following targets:
      
      Target list: #{host_list}
      
      Nmap scan results:
      #{parsed_nmap}
      
      Web screenshot summary:
      #{web_report}
      
      Please analyze these results and identify security issues.
    """,
    temperature=0.3,
    output="initial_analysis",
    // Tasks provided as a separate argument, not in the prompt
    tasks=[deep_scan, vuln_scan, dir_brute, cve_lookup, whois_lookup]
  )
  
  // Generate final security report
  final_report = llm(
    model="mistralai/Mistral-7B-Instruct-v0.2",
    prompt="""
      Based on the reconnaissance and analysis:
      
      Initial analysis: #{initial_analysis}
      
      Additional investigation results (if any):
      #{deep_scan_results || "No deep scan performed"}
      #{vuln_scan_results || "No vulnerability scan performed"}
      #{dir_brute_results || "No directory brute force performed"}
      #{cve_details || "No CVE details looked up"}
      #{whois_results || "No WHOIS lookup performed"}
      
      Provide a summary that would be useful to a penetration tester.
    """,
    temperature=0.2,
    output="report_content"
  )
  
  // Save the report
  save_report = cmd(
    "echo '#{report_content}' > #{OUTPUT_DIR}/security_report.md",
    output="report_path"
  )
  
  // Notify completion
  notify_completion = notify({
    to: "security@example.com",
    subject: "Security Reconnaissance Complete",
    body: "The security reconnaissance for #{TARGET_LIST} is complete.",
    attachment: "#{report_path}"
  })
  
  // Define flow using named tasks
  flow:
    // Start by reading targets
    read_targets >
    
    // Run nmap and eyewitness in parallel
    [nmap_scan, eyewitness] >
    
    // Process results in parallel
    [parse_nmap, process_screenshots] >
    
    // Analyze with LLM (which may run additional tasks)
    analyze_recon >
    
    // Generate and save final report
    final_report > save_report >
    
    // Conditional notification based on scan type
    (scan_type == "thorough" ? notify_completion : null)
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
