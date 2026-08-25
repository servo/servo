import importlib

handler_utils = importlib.import_module(
    "fetch.compression-dictionary.resources.handler_utils")

def main(request, response):
    return handler_utils.create_echo_response(request, response)