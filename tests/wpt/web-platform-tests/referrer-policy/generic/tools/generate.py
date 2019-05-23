#!/usr/bin/env python

import os
import sys

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), '..', '..', '..', 'common', 'security-features', 'tools'))
import generate

class ReferrerPolicyConfig(object):
  def __init__(self):
    self.selection_pattern = '%(delivery_method)s/' + \
                             '%(origin)s/' + \
                             '%(source_protocol)s-%(target_protocol)s/' + \
                             '%(subresource)s/' + \
                             '%(redirection)s/'

    self.test_file_path_pattern = '%(spec_name)s/' + self.selection_pattern + \
                                  '%(name)s.%(source_protocol)s.html'

    self.test_description_template = '''The referrer URL is %(referrer_url)s when a
document served over %(source_protocol)s requires an %(target_protocol)s
sub-resource via %(subresource)s using the %(delivery_method)s
delivery method with %(redirection)s and when
the target request is %(origin)s.'''

    self.test_page_title_template = 'Referrer-Policy: %s'

    self.helper_js = '/referrer-policy/generic/referrer-policy-test-case.sub.js'

    # For debug target only.
    self.sanity_checker_js = '/referrer-policy/generic/sanity-checker.js'
    self.spec_json_js = '/referrer-policy/spec_json.js'

    self.test_case_name = 'ReferrerPolicyTestCase'

    script_directory = os.path.dirname(os.path.abspath(__file__))
    self.spec_directory = os.path.abspath(os.path.join(script_directory, '..', '..'))

  def handleDelivery(self, selection, spec):
    delivery_method = selection['delivery_method']
    delivery_value = spec['referrer_policy']

    meta = ''
    headers = []
    if delivery_value != None:
        if delivery_method == 'meta-referrer':
            meta = \
                '<meta name="referrer" content="%s">' % delivery_value
        elif delivery_method == 'http-rp':
            meta = \
                "<!-- No meta: Referrer policy delivered via HTTP headers. -->"
            headers.append('Referrer-Policy: ' + '%s' % delivery_value)
            # TODO(kristijanburnik): Limit to WPT origins.
            headers.append('Access-Control-Allow-Origin: *')
        elif delivery_method == 'attr-referrer':
            # attr-referrer is supported by the JS test wrapper.
            pass
        elif delivery_method == 'rel-noreferrer':
            # rel=noreferrer is supported by the JS test wrapper.
            pass
        else:
            raise ValueError('Not implemented delivery_method: ' \
                              + delivery_method)
    return {"meta": meta, "headers": headers}


if __name__ == '__main__':
    generate.main(ReferrerPolicyConfig())
