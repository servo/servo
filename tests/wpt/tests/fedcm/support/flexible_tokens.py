import importlib
import json
error_checker = importlib.import_module("fedcm.support.request-params-check")

# Pre-computed token responses
TOKEN_RESPONSES = {
  'string': '{"token": "simple_string_token"}',
  'positive-number': '{"token": 12345}',
  'negative-number': '{"token": -42}',
  'boolean-true': '{"token": true}',
  'boolean-false': '{"token": false}',
  'null': '{"token": null}',
  'zero': '{"token": 0}',
  'float': '{"token": 3.14159}',
  'array': '{"token": ["token1", "token2", "token3"]}',
  'empty-array': '{"token": []}',
  'mixed-array': '{"token": ["string", 123, true, null, {"key": "value"}]}',
  'object': '{"token": {"access_token": "abc123", "token_type": "Bearer", "expires_in": 3600}}',
  'empty-object': '{"token": {}}',
  'nested-object': '{"token": {"user": {"id": "123", "profile": {"name": "Test User", "preferences": {"theme": "dark"}}}}}',
}

def main(request, response):
  request_error = error_checker.tokenCheck(request)
  if request_error:
    return request_error

  response.headers.set(b"Content-Type", b"application/json")
  response.headers.set(b"Access-Control-Allow-Origin", request.headers.get(b"Origin"))
  response.headers.set(b"Access-Control-Allow-Credentials", "true")

  token_type = 'string'  # default

  param_string = request.POST.get(b"params").decode("utf-8")
  param_obj = json.loads(param_string)
  if 'token_type' in param_obj:
    token_type = param_obj['token_type']

  return TOKEN_RESPONSES.get(token_type, TOKEN_RESPONSES['string'])