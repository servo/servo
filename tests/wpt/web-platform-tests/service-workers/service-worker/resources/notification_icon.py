import urlparse

def main(req, res):
  qs_cookie_val = urlparse.parse_qs(req.url_parts.query).get('set-cookie-notification')

  if qs_cookie_val:
    res.set_cookie('notification', qs_cookie_val[0])

  return 'not really an icon'