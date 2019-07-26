#!/usr/bin/env python

import os
import sys

sys.path.insert(
    0,
    os.path.join(
        os.path.dirname(os.path.abspath(__file__)), '..', '..', '..', 'common',
        'security-features', 'tools'))
import generate


class ReferrerPolicyConfig(object):
    def __init__(self):
        self.selection_pattern = '%(delivery_type)s/' + \
                                 '%(origin)s/' + \
                                 '%(source_scheme)s/' + \
                                 '%(subresource)s/' + \
                                 '%(redirection)s/'

        self.test_file_path_pattern = '%(spec_name)s/' + self.selection_pattern + \
                                      '%(name)s.%(source_scheme)s.html'

        self.test_description_template = '''The referrer URL is %(expectation)s when a
document served over %(source_scheme)s requires a
sub-resource via %(subresource)s using the %(delivery_type)s
delivery method with %(redirection)s and when
the target request is %(origin)s.'''

        self.test_page_title_template = 'Referrer-Policy: %s'

        self.helper_js = '/referrer-policy/generic/referrer-policy-test-case.sub.js'

        # For debug target only.
        self.sanity_checker_js = '/referrer-policy/generic/sanity-checker.js'
        self.spec_json_js = '/referrer-policy/spec_json.js'

        self.test_case_name = 'ReferrerPolicyTestCase'

        script_directory = os.path.dirname(os.path.abspath(__file__))
        self.spec_directory = os.path.abspath(
            os.path.join(script_directory, '..', '..'))

    def handleDelivery(self, selection, spec):
        delivery_type = selection['delivery_type']
        delivery_value = selection['delivery_value']

        meta = ''
        headers = []
        if delivery_value != None:
            if delivery_type == 'meta':
                meta = \
                    '<meta name="referrer" content="%s">' % delivery_value
            elif delivery_type == 'http-rp':
                meta = \
                    "<!-- No meta: Referrer policy delivered via HTTP headers. -->"
                headers.append('Referrer-Policy: ' + '%s' % delivery_value)
                # TODO(kristijanburnik): Limit to WPT origins.
                headers.append('Access-Control-Allow-Origin: *')
            elif delivery_type == 'attr':
                # attr-referrer is supported by the JS test wrapper.
                pass
            elif delivery_type == 'rel-noref':
                # rel=noreferrer is supported by the JS test wrapper.
                pass
            else:
                raise ValueError('Not implemented delivery_type: ' \
                                  + delivery_type)
        return {"meta": meta, "headers": headers}


if __name__ == '__main__':
    generate.main(ReferrerPolicyConfig())
