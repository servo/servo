# 'import credential-management.support.fedcm.keys' does not work.
import importlib
keys = importlib.import_module("credential-management.support.fedcm.keys")
error_checker = importlib.import_module("credential-management.support.fedcm.request-params-check")

def main(request, response):
  request_error = error_checker.clientMetadataCheck(request)
  if (request_error):
    return request_error

  counter = request.server.stash.take(keys.CLIENT_METADATA_COUNTER_KEY)
  try:
    counter = int(counter) + 1
  except (TypeError, ValueError):
    counter = 1

  request.server.stash.put(keys.CLIENT_METADATA_COUNTER_KEY, str(counter).encode())

  response.headers.set(b"Content-Type", b"application/json")

  return """
{{
  "privacy_policy_url": "https://privacypolicy{0}.com"
}}
""".format(str(counter))
