import logging

logger = logging.getLogger(__name__)

def run_step(context):
    context.assert_key_has_value(key='bar', caller=__name__)

    print('GOT IT BOSS', context.get_formatted('bar'))