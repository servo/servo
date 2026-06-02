"""Utility functions shared across multiple endpoints."""
from collections import namedtuple
from urllib.parse import unquote_plus, urlparse

def fail(response, body):
    """Sets up response to fail with the provided response body.

    Args:
      response: the wptserve Response that was passed to main
      body: the HTTP response body to use
    """
    response.status = (400, "Bad Request")
    response.headers.set(b"Content-Type", b"text/plain")
    response.content = body

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

def attach_origin_and_credentials_headers(request, response):
  """Attaches Access-Control-Allow-Origin and Access-Control-Allow-Credentials
  response headers to a response, if the request indicates they're needed.
  Only intended for internal use.

  Args:
    request: the wptserve Request that was passed to main
    response: the wptserve Response that was passed to main
  """
  if b"origin" in request.headers:
    response.headers.set(b"Access-Control-Allow-Origin",
                        request.headers.get(b"origin"))

  if b"credentials" in request.headers:
    response.headers.set(b"Access-Control-Allow-Credentials",
                        request.headers.get(b"credentials"))

def handle_cors_headers_fail_if_preflight(request, response):
  """Adds CORS headers if necessary. In the case of CORS preflights, generates
  a failure response. To be used when CORS preflights are not expected.

  Args:
    request: the wptserve Request that was passed to main
    response: the wptserve Response that was passed to main

  Returns True if the request is a CORS preflight, in which case the calling
  function should immediately return.
  """
  # Handle CORS preflight requests.
  if request.method == u"OPTIONS":
    fail(response, "CORS preflight unexpectedly received.")
    return True

  # Append CORS headers if needed
  attach_origin_and_credentials_headers(request, response)
  return False

def handle_cors_headers_and_preflight(request, response):
  """Applies CORS logic, either adding CORS headers to response or generating
  an entire response to preflights.

  Args:
    request: the wptserve Request that was passed to main
    response: the wptserve Response that was passed to main

  Returns True if the request is a CORS preflight, in which case the calling
  function should immediately return.
  """
  # Append CORS headers if needed
  attach_origin_and_credentials_headers(request, response)

  # Handle CORS preflight requests.
  if not request.method == u"OPTIONS":
    return False

  if not b"Access-Control-Request-Method" in request.headers:
    fail(response, "Failed to get access-control-request-method in preflight!")
    return True

  if not b"Access-Control-Request-Headers" in request.headers:
    fail(response, "Failed to get access-control-request-headers in preflight!")
    return True

  response.headers.set(b"Access-Control-Allow-Methods",
                        request.headers[b"Access-Control-Request-Method"])

  response.headers.set(b"Access-Control-Allow-Headers",
                        request.headers[b"Access-Control-Request-Headers"])

  response.status = (204, b"No Content")
  return True

def decode_trusted_scoring_signals_params(request):
  """Decodes query parameters to trusted query params handler.

  Args:
    request: the wptserve Request that was passed to main

  If successful, returns a named tuple TrustedScoringSignalsParams decoding the
  various expected query fields, as a hostname,  plus a field urlLists which is a list of
  {type: <render URL type>, urls: <render URL list>} pairs, where <render URL type> is
  one of the two render URL dictionary keys used in the response ("renderURLs" or
  "adComponentRenderURLs"). May be of length 1 or 2, depending on whether there
  are any component URLs.

  On failure, throws a ValueError with a message.
  """
  TrustedScoringSignalsParams = namedtuple(
      'TrustedScoringSignalsParams', ['hostname', 'urlLists'])

  hostname = None
  renderUrls = None
  adComponentRenderURLs = None
  urlLists = []

  # Manually parse query params. Can't use request.GET because it unescapes as well as splitting,
  # and commas mean very different things from escaped commas.
  for param in request.url_parts.query.split("&"):
      pair = param.split("=", 1)
      if len(pair) != 2:
          raise ValueError("Bad query parameter: " + param)
      # Browsers should escape query params consistently.
      if "%20" in pair[1]:
          raise ValueError("Query parameter should escape using '+': " + param)

      # Hostname can't be empty. The empty string can be a key or interest group name, though.
      if pair[0] == "hostname" and hostname == None and len(pair[1]) > 0:
          hostname = pair[1]
          continue
      if pair[0] == "renderUrls" and renderUrls == None:
          renderUrls = list(map(unquote_plus, pair[1].split(",")))
          urlLists.append({"type":"renderURLs", "urls":renderUrls})
          continue
      if pair[0] == "adComponentRenderUrls" and adComponentRenderURLs == None:
          adComponentRenderURLs = list(map(unquote_plus, pair[1].split(",")))
          urlLists.append({"type":"adComponentRenderURLs", "urls":adComponentRenderURLs})
          continue
      # Ignore the various creative scanning params; they're expected, but we
      # don't parse them here.
      if (pair[0] == 'adCreativeScanningMetadata' or
            pair[0] == 'adComponentCreativeScanningMetadata' or
            pair[0] == 'adSizes' or
            pair[0] == 'adComponentSizes' or
            pair[0] == 'adBuyer' or
            pair[0] == 'adComponentBuyer' or
            pair[0] == 'adBuyerAndSellerReportingIds'):
          continue
      raise ValueError("Unexpected query parameter: " + param)

  # "hostname" and "renderUrls" are mandatory.
  if not hostname:
      raise ValueError("hostname missing")
  if not renderUrls:
      raise ValueError("renderUrls missing")

  return TrustedScoringSignalsParams(hostname, urlLists)

def decode_render_url_signals_params(renderUrl):
  """Decodes signalsParams field encoded inside a renderURL.

  Args: renderUrl to extract signalsParams from.

  Returns an array of fields in signal params string.
  """
  signalsParams = None
  for param in urlparse(renderUrl).query.split("&"):
    pair = param.split("=", 1)
    if len(pair) != 2:
        continue
    if pair[0] == "signalsParams":
        if signalsParams != None:
            raise ValueError("renderUrl has multiple signalsParams: " + renderUrl)
        signalsParams = pair[1]

  if signalsParams is None:
    return []

  signalsParams = unquote_plus(signalsParams)
  return signalsParams.split(",")
