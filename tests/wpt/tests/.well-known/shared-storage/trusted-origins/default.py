"""Endpoint to get shared storage trusted origins."""
from importlib import import_module

trusted_origins = import_module('shared-storage.resources.trusted-origins')

def main(request, response):
  trusted_origins.get_json(request, response)
