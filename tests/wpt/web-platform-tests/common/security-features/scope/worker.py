import os, sys, json

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import util


def main(request, response):
  policyDeliveries = json.loads(request.GET.first('policyDeliveries', '[]'))
  worker_type = request.GET.first('type', 'classic')
  commonjs_url = '%s://%s:%s/common/security-features/resources/common.sub.js' % (
      request.url_parts.scheme, request.url_parts.hostname,
      request.url_parts.port)
  if worker_type == 'classic':
    import_line = 'importScripts("%s");' % commonjs_url
  else:
    import_line = 'import "%s";' % commonjs_url

  maybe_additional_headers = {}
  error = ''
  for delivery in policyDeliveries:
    if delivery['deliveryType'] == 'meta':
      error = '<meta> cannot be used in WorkerGlobalScope'
    elif delivery['deliveryType'] == 'http-rp':
      if delivery['key'] == 'referrerPolicy':
        maybe_additional_headers['Referrer-Policy'] = delivery['value']
      elif delivery['key'] == 'mixedContent' and delivery['value'] == 'opt-in':
        maybe_additional_headers['Content-Security-Policy'] = 'block-all-mixed-content'
      elif delivery['key'] == 'upgradeInsecureRequests' and delivery['value'] == 'upgrade':
        maybe_additional_headers['Content-Security-Policy'] = 'upgrade-insecure-requests'
      else:
        error = 'invalid delivery key for http-rp: %s' % delivery['key']
    else:
      error = 'invalid deliveryType: %s' % delivery['deliveryType']

  handler = lambda: util.get_template('worker.js.template') % ({
      'import': import_line,
      'error': error
  })
  util.respond(
      request,
      response,
      payload_generator=handler,
      content_type='text/javascript',
      maybe_additional_headers=maybe_additional_headers)
