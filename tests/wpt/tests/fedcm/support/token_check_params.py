import importlib
import json
error_checker = importlib.import_module("fedcm.support.request-params-check")

def main(request, response):
  request_error = error_checker.tokenCheck(request)
  if (request_error):
    return request_error

  param_string = request.POST.get(b"params").decode("utf-8")
  param_obj = json.loads(param_string)
  if param_obj['a string'] != 'a value':
    return (550, [], "Incorrect parameters")

  response.headers.set(b"Content-Type", b"application/json")
  response.headers.set(b"Access-Control-Allow-Origin", request.headers.get(b"Origin"))
  response.headers.set(b"Access-Control-Allow-Credentials", "true")

  return "{\"token\": \"token\"}"
