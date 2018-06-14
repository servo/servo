# flake8: noqa (not ideal, but nicer than adding noqa: F401 to every line!)
import openssl
import pregenerated
from base import NoSSLEnvironment
from openssl import OpenSSLEnvironment
from pregenerated import PregeneratedSSLEnvironment

environments = {"none": NoSSLEnvironment,
                "openssl": OpenSSLEnvironment,
                "pregenerated": PregeneratedSSLEnvironment}
