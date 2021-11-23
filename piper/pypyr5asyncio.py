import asyncio

import logging

from pypyr import pipelinerunner

from rich.console import Console
from rich.logging import RichHandler


FORMAT = "%(message)s"
logging.basicConfig(
    level=25, format=FORMAT, datefmt="[%X]", handlers=[RichHandler()]
)
console = Console()


def run_pipeline(dict_in):
    # this is blocking
    # returns the context as it is after pipeline completes
    # this context is unique to this specific pipeline run attempt
    return pipelinerunner.run(pipeline_name='pipelines/test-pipe',
                              dict_in=dict_in)


async def main():
    """async entrypoint"""
    loop = asyncio.get_running_loop()

    futures = []

    # somewhat arbitrarily create 5X pypyr run instructions/futures
    for i in range(5):
        # initialize context with this:
        dict_in = {'finished': False,
                   'progress': 0,
                   'arbkey': 'pipe',
                   'anotherkey': 'song',
                   'runid': i
                   }

        # this runs on the default loop's executor
        # you maybe want to create a custom threadpool instead
        futures.append(loop.run_in_executor(None, run_pipeline, dict_in))

    # run the futures all concurrently
    console.log("starting pipelines. . .")
    results = await asyncio.gather(*futures)

    # each result is the output context from each individual pipeline run
    for result in results:
        # printing for the giggles, but you can merge/update etc. here to
        # do something useful instead
        console.log(result)

asyncio.run(main())

console.log(':smiley: done')
