import random
import time

from wptserve.utils import isomorphic_decode


"""
This script serves
"""

def main(request, response):
  inlineOrExternal = request.GET.first(b"inlineOrExternal", b"null")
  hasBlockingStylesheet = request.GET.first(b"hasBlockingStylesheet", b"true") == b"true"
  result = request.GET.first(b"result", b"success")
  type = u"text/javascript" if request.GET.first(b"type", b"classic") == b"classic" else u"module"

  response.headers.set(b"Content-Type", b"text/html; charset=utf-8")
  response.headers.set(b"Transfer-Encoding", b"chunked")
  response.write_status_headers()

  # Step 1: Start parsing.
  body = u"""<!DOCTYPE html>
    <head>
      <script>
        parent.postMessage("fox", "*");
      </script>
  """

  if hasBlockingStylesheet:
    body += u"""
        <link rel="stylesheet" href="slow-flag-setter.py?result=css&cache=%f">
      """ % random.random()

  body += u"""
    </head>
    <body>
  """

  if inlineOrExternal == b"inline" or inlineOrExternal == b"external" or inlineOrExternal == b"empty-src":
        body += u"""
      <streaming-element>
    """

  # Trigger DOM processing
  body += u"A" * 100000

  response.writer.write(u"%x\r\n" % len(body))
  response.writer.write(body)
  response.writer.write(u"\r\n")
  response.writer.flush()

  body = u""

  if inlineOrExternal == b"inline":
    time.sleep(1)
    body += u"""
        <script id="s1" type="%s"
                onload="scriptOnLoad()"
                onerror="scriptOnError(event)">
        if (!window.readyToEvaluate) {
          window.didExecute = "executed too early";
        } else {
          window.didExecute = "executed";
        }
    """ % type
    if result == b"parse-error":
      body += u"1=2 parse error\n"

    body += u"""
        </script>
      </streaming-element>
    """
  elif inlineOrExternal == b"external":
    time.sleep(1)
    body += u"""
        <script id="s1" type="%s"
                src="slow-flag-setter.py?result=%s&cache=%s"
                onload="scriptOnLoad()"
                onerror="scriptOnError(event)"></script>
      </streaming-element>
    """ % (type, isomorphic_decode(result), random.random())
  elif inlineOrExternal == b"empty-src":
    time.sleep(1)
    body += u"""
        <script id="s1" type="%s"
                src=""
                onload="scriptOnLoad()"
                onerror="scriptOnError(event)"></script>
      </streaming-element>
    """ % (type,)

  #        // if readyToEvaluate is false, the script is probably
  #       // wasn't blocked by stylesheets as expected.

  # Trigger DOM processing
  body += u"B" * 100000

  response.writer.write(u"%x\r\n" % len(body))
  response.writer.write(body)
  response.writer.write(u"\r\n")

  response.writer.write(u"0\r\n")
  response.writer.write(u"\r\n")
  response.writer.flush()
