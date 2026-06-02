"""Endpoint to get interest group cross-origin permissions."""
from importlib import import_module

permissions = import_module('fledge.tentative.resources.permissions')


def main(request, response):
  permissions.get_permissions(request, response)
