"""Utility functions shared across multiple endpoints."""


def headers_to_ascii(headers):
  """Converts a header map with binary values to one with ASCII values.

  Takes a map of header names to list of values that are all binary strings
  and returns an otherwise identical map where keys and values have both been
  converted to ASCII strings.

  Args:
    headers: header map from binary key to binary value

  Returns header map from ASCII string key to ASCII string value
  """
  header_map = {}
  for pair in headers.items():
      values = []
      for value in pair[1]:
          values.append(value.decode("ASCII"))
      header_map[pair[0].decode("ASCII")] = values
  return header_map


def handle_cors_headers_and_preflight(request, response):
  """Applies CORS logic common to many entrypoints.

  Args:
    request: the wptserve Request that was passed to main
    response: the wptserve Response that was passed to main

  Returns True if the request is a CORS preflight, which is entirely handled by
  this function, so that the calling function should immediately return.
  """
  # Append CORS headers if needed
  if b"origin" in request.headers:
    response.headers.set(b"Access-Control-Allow-Origin",
                        request.headers.get(b"origin"))

  if b"credentials" in request.headers:
    response.headers.set(b"Access-Control-Allow-Credentials",
                        request.headers.get(b"credentials"))

  # Handle CORS preflight requests.
  if not request.method == u"OPTIONS":
    return False

  if not b"Access-Control-Request-Method" in request.headers:
    response.status = (400, b"Bad Request")
    response.headers.set(b"Content-Type", b"text/plain")
    response.content = "Failed to get access-control-request-method in preflight!"
    return True

  if not b"Access-Control-Request-Headers" in request.headers:
    response.status = (400, b"Bad Request")
    response.headers.set(b"Content-Type", b"text/plain")
    response.content = "Failed to get access-control-request-headers in preflight!"
    return True

  response.headers.set(b"Access-Control-Allow-Methods",
                        request.headers[b"Access-Control-Request-Method"])

  response.headers.set(b"Access-Control-Allow-Headers",
                        request.headers[b"Access-Control-Request-Headers"])

  response.status = (204, b"No Content")
  return True
