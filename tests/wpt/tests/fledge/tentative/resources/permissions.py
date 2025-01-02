"""Methods for the interest group cross-origin permissions endpoint."""
import json
import re

from fledge.tentative.resources import fledge_http_server_util

SUBDOMAIN_WWW = 'www'
SUBDOMAIN_WWW1 = 'www1'
SUBDOMAIN_WWW2 = 'www2'
SUBDOMAIN_FRENCH = 'élève'.encode('idna').decode()
SUBDOMAIN_JAPANESE = '天気の良い日'.encode('idna').decode()
ALL_SUBDOMAINS = [SUBDOMAIN_WWW, SUBDOMAIN_WWW1, SUBDOMAIN_WWW2,
                  SUBDOMAIN_FRENCH, SUBDOMAIN_JAPANESE]

def get_permissions(request, response):
  """Returns JSON object containing interest group cross-origin permissions.

  The structure returned is described in more detail at
  https://github.com/WICG/turtledove/blob/main/FLEDGE.md#13-permission-delegation.
  This correctly handles requests issued in CORS mode.

  This .well-known is fetched at the origin of the interest group's owner, and
  specifies as a URL parameter the origin of frame that's attempting to join or
  leave that interest group.

  This is implemented such that the origin of the frame is ignored altogether,
  and the determination of which operations are allowed depends strictly on the
  origin of the interest group owner, and specifically on the subdomain of the
  origin of the interest group owner. wptserve serves each of its two domains
  at both the raw domain and each of five subdomains.

  - www: disallows both join and leave
  - www1: allows join, but not leave
  - www2: allows leave, but not join
  - 天気の良い日 / élève: allow both join and leave
  - anything else (including no subdomain): returns a 404
  """
  if fledge_http_server_util.handle_cors_headers_and_preflight(request, response):
    return

  first_domain_label = re.search(r"[^.]*", request.url_parts.netloc).group(0)
  if first_domain_label not in ALL_SUBDOMAINS:
    response.status = (404, b"Not Found")
    response.content = "Not Found"
    return

  response.status = (200, b"OK")
  response.headers.set(b"Content-Type", b"application/json")
  response.content = json.dumps({
      "joinAdInterestGroup": first_domain_label in [
          SUBDOMAIN_WWW1, SUBDOMAIN_FRENCH, SUBDOMAIN_JAPANESE],
      "leaveAdInterestGroup": first_domain_label in [
          SUBDOMAIN_WWW2, SUBDOMAIN_FRENCH, SUBDOMAIN_JAPANESE],
  })
