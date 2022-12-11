# 'import credential-management.support.fedcm.keys' does not work.
import importlib
keys = importlib.import_module("credential-management.support.fedcm.keys")

def main(request, response):
  if (request.GET.get(b'skip_checks', b'0') != b'1'):
    if len(request.cookies) > 0:
      return (530, [], "Cookie should not be sent to this endpoint")
    if request.headers.get(b"Accept") != b"application/json":
      return (531, [], "Wrong Accept")
    if request.headers.get(b"Sec-Fetch-Dest") != b"webidentity":
      return (532, [], "Wrong Sec-Fetch-Dest header")
    if request.headers.get(b"Referer"):
      return (533, [], "Should not have Referer")
    if not request.headers.get(b"Origin"):
      return (534, [], "Missing Origin")

  counter = request.server.stash.take(keys.CLIENT_METADATA_COUNTER_KEY)
  try:
    counter = int(counter) + 1
  except (TypeError, ValueError):
    counter = 1

  request.server.stash.put(keys.CLIENT_METADATA_COUNTER_KEY, str(counter).encode())

  return """
{{
  "privacy_policy_url": "https://privacypolicy{0}.com"
}}
""".format(str(counter))
