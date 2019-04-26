import os, sys, json

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import util


def main(request, response):
  policyDeliveries = json.loads(request.GET.first("policyDeliveries", "[]"))
  maybe_additional_headers = {}
  meta = ''
  error = ''
  for delivery in policyDeliveries:
    if delivery['deliveryType'] == 'meta':
      if delivery['key'] == 'referrerPolicy':
        meta += '<meta name="referrer" content="%s">' % delivery['value']
      else:
        error = 'invalid delivery key'
    elif delivery['deliveryType'] == 'http-rp':
      if delivery['key'] == 'referrerPolicy':
        maybe_additional_headers['Referrer-Policy'] = delivery['value']
      else:
        error = 'invalid delivery key'
    else:
      error = 'invalid deliveryType'

  handler = lambda: util.get_template("document.html.template") % ({
      "meta": meta,
      "error": error
  })
  util.respond(
      request,
      response,
      payload_generator=handler,
      content_type="text/html",
      maybe_additional_headers=maybe_additional_headers)
