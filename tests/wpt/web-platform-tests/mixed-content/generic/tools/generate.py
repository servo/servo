#!/usr/bin/env python

import os
import sys

sys.path.insert(
    0,
    os.path.join(
        os.path.dirname(os.path.abspath(__file__)), '..', '..', '..', 'common',
        'security-features', 'tools'))
import generate


class MixedContentConfig(object):
    def __init__(self):
        self.selection_pattern = '%(subresource)s/' + \
                                 '%(delivery_type)s/' + \
                                 '%(delivery_value)s/' + \
                                 '%(origin)s/' + \
                                 'top-level/' + \
                                 '%(redirection)s/'

        self.test_file_path_pattern = self.selection_pattern + \
                                      '%(spec_name)s/' + \
                                      '%(name)s.%(source_scheme)s.html'

        self.test_description_template = '''delivery_type: %(delivery_type)s
delivery_value: %(delivery_value)s
origin: %(origin)s
source_scheme: %(source_scheme)s
context_nesting: top-level
redirection: %(redirection)s
subresource: %(subresource)s
expectation: %(expectation)s
'''

        self.test_page_title_template = 'Mixed-Content: %s'

        self.helper_js = '/mixed-content/generic/mixed-content-test-case.js?pipe=sub'

        # For debug target only.
        self.sanity_checker_js = '/mixed-content/generic/sanity-checker.js'
        self.spec_json_js = '/mixed-content/spec_json.js'

        self.test_case_name = 'MixedContentTestCase'

        script_directory = os.path.dirname(os.path.abspath(__file__))
        self.spec_directory = os.path.abspath(
            os.path.join(script_directory, '..', '..'))


if __name__ == '__main__':
    generate.main(MixedContentConfig())
