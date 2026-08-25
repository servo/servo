import importlib
registerDictionary = importlib.import_module("fetch.compression-dictionary.resources.register-dictionary")

def main(request, response):
    return registerDictionary.main(request, response)
