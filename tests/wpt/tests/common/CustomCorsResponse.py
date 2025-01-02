import json

def main(request, response):
  '''Handler for getting an HTTP response customised by the given query
  parameters.

  The returned response will have
    - HTTP headers defined by the 'headers' query parameter
      - Must be a serialized JSON dictionary mapping header names to header
        values
    - HTTP status code defined by the 'status' query parameter
      - Must be a positive serialized JSON integer like the string '200'
    - Response content defined by the 'content' query parameter
      - Must be a serialized JSON string representing the desired response body
  '''
  def query_parameter_or_default(param, default):
    return request.GET.first(param) if param in request.GET else default

  headers = json.loads(query_parameter_or_default(b'headers', b'"{}"'))
  for k, v in headers.items():
    response.headers.set(k, v)

  # Note that, in order to have out-of-the-box support for tests that don't call
  #   setup({'allow_uncaught_exception': true})
  # we return a no-op JS payload. This approach will avoid syntax errors in
  # script resources that would otherwise cause the test harness to fail.
  response.content = json.loads(query_parameter_or_default(b'content',
    b'"/* CustomCorsResponse.py content */"'))
  response.status_code = json.loads(query_parameter_or_default(b'status',
    b'200'))
