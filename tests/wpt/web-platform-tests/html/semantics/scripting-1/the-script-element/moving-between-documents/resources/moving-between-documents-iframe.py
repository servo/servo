import random
import time

"""
This script serves
"""

def main(request, response):
  inlineOrExternal = request.GET.first("inlineOrExternal", "null")
  hasBlockingStylesheet = request.GET.first("hasBlockingStylesheet", "true") == "true"
  result = request.GET.first("result", "success")
  type = "text/javascript" if request.GET.first("type", "classic") == "classic" else "module"

  response.headers.set("Content-Type", "text/html; charset=utf-8")
  response.headers.set("Transfer-Encoding", "chunked")
  response.write_status_headers()

  # Step 1: Start parsing.
  body = """<!DOCTYPE html>
    <head>
      <script>
        parent.postMessage("fox", "*");
      </script>
  """

  if hasBlockingStylesheet:
    body += """
        <link rel="stylesheet" href="slow-flag-setter.py?result=css&cache=%f">
      """ % random.random()

  body += """
    </head>
    <body>
  """

  if inlineOrExternal == "inline" or inlineOrExternal == "external":
    body += """
      <streaming-element>
    """

  # Trigger DOM processing
  body += "A" * 100000

  response.writer.write("%x\r\n" % len(body))
  response.writer.write(body)
  response.writer.write("\r\n")
  response.writer.flush()

  body = ""

  if inlineOrExternal == "inline":
    time.sleep(1)
    body += """
        <script id="s1" type="%s"
                onload="tScriptLoadEvent.unreached_func('onload')"
                onerror="scriptOnError(event)">
        if (!window.readyToEvaluate) {
          window.didExecute = "executed too early";
        } else {
          window.didExecute = "executed";
        }
    """ % (type)
    if result == "parse-error":
      body += "1=2 parse error\n"

    body += """
        </script>
      </streaming-element>
    """
  elif inlineOrExternal == "external":
    time.sleep(1)
    body += """
        <script id="s1" type="%s"
                src="slow-flag-setter.py?result=%s&cache=%s"
                onload="tScriptLoadEvent.unreached_func('onload')"
                onerror="scriptOnError(event)"></script>
      </streaming-element>
    """ % (type, result, random.random())

  #        // if readyToEvaluate is false, the script is probably
  #       // wasn't blocked by stylesheets as expected.

  # Trigger DOM processing
  body += "B" * 100000

  response.writer.write("%x\r\n" % len(body))
  response.writer.write(body)
  response.writer.write("\r\n")

  response.writer.write("0\r\n")
  response.writer.write("\r\n")
  response.writer.flush()
