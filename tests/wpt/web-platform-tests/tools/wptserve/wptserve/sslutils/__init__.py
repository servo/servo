from .base import NoSSLEnvironment
from .openssl import OpenSSLEnvironment
from .pregenerated import PregeneratedSSLEnvironment

environments = {"none": NoSSLEnvironment,
                "openssl": OpenSSLEnvironment,
                "pregenerated": PregeneratedSSLEnvironment}
