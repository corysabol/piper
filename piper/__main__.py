import time
import os
import argparse
import threading
import logging
from pathlib import Path

from pypyr import pipelinerunner
from pypyr import context
from pypyr.pypeloaders import fileloader
from pypyr.stepsrunner import StepsRunner
from pypyr.dsl import Step

from rich.console import Console
from rich.table import Table
from rich.progress import track
from rich.progress import Progress
from rich import inspect
from rich.logging import RichHandler
from rich import print

from tinydb import TinyDB, Query

import ast

# TODO: allow the user to specify / select a DB file to 
# use for a given pipeline
db = TinyDB('./db.json')

host = {
    'ip': '',
    'hostname': '',
    'steps': {},
    'services': {},
    'last_seen': ''
}

FORMAT = "%(message)s"
logging.basicConfig(
    level=25, format=FORMAT, datefmt="[%X]", handlers=[RichHandler()]
)
console = Console()

def pipeline_thread_function(shared_context):
    pipelinerunner.prepare_and_run(
        pipeline_name='pipelines/test-pipe',
        working_dir=os.getcwd(),
        context=shared_context,
        parse_input=False
        )   

shared_context = context.Context({
    'finished': False,
    'progress': 0, 
    'arbkey':'pipe', 
    'anotherkey':'song',
    'logger': console.log,
    'db': db
}) 

pipeline_definition = fileloader.get_pipeline_definition('pipelines/test-pipe', Path(os.getcwd()))
stepsrunner = StepsRunner(pipeline_definition, shared_context) 
steps = stepsrunner.get_pipeline_steps('steps')
step_count = 0

with console.status('[bold green]Hacking stuff...', spinner='pong'):
    for step in steps:
        console.log(f"[bold magenta]{step.get('name').upper()}:")
        if step.get('comment', None):
            console.log(f"[cyan]{step.get('comment')}")

        step_instance = Step(step, stepsrunner)
        step_instance.run_step(shared_context)
        
        if shared_context.get('cmdOut', None):
            cmd_out = shared_context.get('cmdOut')
            shared_context['cmdOut'] = ''
            console.log(cmd_out['stdout'])

        console.log(f':white_check_mark: [green]{step["name"]} compelete\n')
        step_count += 1

console.log(':smiley: [bold green]done[/bold green] :confetti_ball:')