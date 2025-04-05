import importlib
header_helpers = importlib.import_module("storage-access-api.resources.header-helpers")

def main(request, response):
  return header_helpers.make_response_body(request.GET)
