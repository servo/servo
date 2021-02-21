import six


def cast_env(env):
    """Encode all the environment values as the appropriate type.
    This assumes that all the data is or can be represented as UTF8"""

    return {six.ensure_str(key): six.ensure_str(value) for key, value in env.items()}
