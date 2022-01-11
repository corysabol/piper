import logging
import ast

from pypyr.steps.dsl.cmd import CmdStep

logger = logging.getLogger(__name__)

def run_step(context):
    logger.debug("started")

    context.assert_key_has_value(key='docker', caller=__name__)
    context.assert_key_has_value(key='flags', caller=__name__)
    context.assert_key_has_value(key='persist', caller=__name__)
    context.assert_key_has_value(key='hosts', caller=__name__)

    # wrap the pypyr cmd step in our own shitty logic
    if context.get('docker', False):
        # execute rustscan using docker
        context['cmd'] = {
            'run': 'docker run -it --rm --name rustscan rustscan/rustscan --accessible -a {hosts} {flags}',
            'save': True
        }
    else:
        context['cmd'] = {
            'run': 'rustscan --accessible -a {hosts} {flags}',
            'save': True
        }
    CmdStep(name=__name__, context=context).run_step(is_shell=False)

    # save is always true, but was it greppable or not? the parser will be different
    if context.get('persist', False):
        if '--greppable' in context['flags']:
            logger.debug('greppable output, persisting...')
            # persist to db
            cmd_out = context.get('cmdOut')
            stdout_split = cmd_out['stdout'].split('\n')

            for h in stdout_split:
                context['db'].table(__name__).insert({
                    h.split()[0]: {
                        'ports': ast.literal_eval(h.split()[2])
                    }
                })
        else:
            None # TODO implement nmap parser / call nmap parser?

    # parse results 
    logger.debug("done")