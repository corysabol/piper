# piper
Piper is super glue for your shell and Lua workflows.

Piper is a tool aimed at allowing you to author repeatable pipeline using a mixture of shell scripting and Lua, and run them locally or against a Piper remote agent via gRPC.

## Core Concepts

Piper is built on the following core concepts:

### 📡 Remote execution
### 📝 Persistence
### 📬 Notifications
### 🚇 Pipelines

#### Example Pipeline
```yaml
name: Foo
author: Pippins
description: |
  This is just a test pipeline.
tasks:
  - name: Run a shell command
    comment: echo something
    task: cmd
    args:
      cmd: "echo \"hi from a pipeline task\""
  - name: Set global variable
    comment: Set a global variable in the ctx object
    task: script
    args:
      script: |
        print("Setting ctx value from python task")
        ctx["test"] = "This is a test"
  - task: set_var
    args:
      type: number
      var: bar
      val: 42
  - task: script
    args:
      script: |
        print("Getting bar")
        print(ctx["bar"])
        print("Use piper string interpolation to get bar - #{bar}")
  - name: test variable interpolation
    comment: testing var interpolation using \#{var}
    task: cmd
    args:
      cmd: echo '#{bar}'
  - name: Access ctx object
    comment: foo
    task: script
    args:
      script: |
        print(f'Context from Python3 {ctx}')
```

## Basic Usage
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
