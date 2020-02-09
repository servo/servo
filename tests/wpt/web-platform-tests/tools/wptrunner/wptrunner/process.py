import sys

import six


def cast_env(env):
    """Encode all the environment values as the appropriate type for each Python version
    This assumes that all the data is or can be represented as UTF8"""

    env_type = six.ensure_binary if sys.version_info[0] < 3 else six.ensure_str
    return {env_type(key): env_type(value) for key, value in six.iteritems(env)}
