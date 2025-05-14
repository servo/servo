import os
from wptserve.pipes import template

# Returns the executor HTML (as bytes).
# `additional_script` (str) is inserted to the JavaScript before the executor.
def get_executor_html(request, additional_script):
  content = template(
    request,
    open(os.path.join(os.path.dirname(__file__), "executor.sub.html"), "rb").read())

  # Insert an additional script at the head of script before Executor.
  content = content.replace(
      b'<script nonce="abc">',
      b'<script nonce="abc">' + additional_script.encode('utf-8'))

  return content
