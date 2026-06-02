# 'import fedcm.support.keys' does not work.
import importlib
keys = importlib.import_module("fedcm.support.keys")
error_checker = importlib.import_module("fedcm.support.request-params-check")

def main(request, response):
  request_error = error_checker.clientMetadataCheck(request)
  if (request_error):
    return request_error

  if not request.GET.get(b"top_frame_origin"):
    return (560, [], "Missing top_frame_origin")

  response.headers.set(b"Content-Type", b"application/json")

  return """
{
  "client_is_third_party_to_top_frame_origin": true
}
"""
