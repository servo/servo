from .base import NoSSLEnvironment
from .openssl import OpenSSLEnvironment
from .pregenerated import PregeneratedSSLEnvironment

environments = {"none": NoSSLEnvironment,
                "openssl": OpenSSLEnvironment,
                "pregenerated": PregeneratedSSLEnvironment}


def get_cls(name):
    try:
        return environments[name]
    except KeyError:
        raise ValueError("%s is not a vaid ssl type." % name)
