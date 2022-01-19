from urllib.parse import parse_qs

from wptserve.utils import isomorphic_encode

def main(req, res):
  qs_cookie_val = parse_qs(req.url_parts.query).get(u'set-cookie-notification')

  if qs_cookie_val:
    res.set_cookie(b'notification', isomorphic_encode(qs_cookie_val[0]))

  return b'not really an icon'
