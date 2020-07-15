import os, sys, json

from wptserve.utils import isomorphic_decode, isomorphic_encode

sys.path.insert(0, os.path.dirname(os.path.abspath(isomorphic_decode(__file__))))
import util

def main(request, response):
  policyDeliveries = json.loads(request.GET.first(b"policyDeliveries", b"[]"))
  maybe_additional_headers = {}
  meta = u''
  error = u''
  for delivery in policyDeliveries:
    if delivery[u'deliveryType'] == u'meta':
      if delivery[u'key'] == u'referrerPolicy':
        meta += u'<meta name="referrer" content="%s">' % delivery[u'value']
      else:
        error = u'invalid delivery key'
    elif delivery[u'deliveryType'] == u'http-rp':
      if delivery[u'key'] == u'referrerPolicy':
        maybe_additional_headers[b'Referrer-Policy'] = isomorphic_encode(delivery[u'value'])
      else:
        error = u'invalid delivery key'
    else:
      error = u'invalid deliveryType'

  handler = lambda: util.get_template(u"document.html.template") % ({
      u"meta": meta,
      u"error": error
  })
  util.respond(
      request,
      response,
      payload_generator=handler,
      content_type=b"text/html",
      maybe_additional_headers=maybe_additional_headers)
