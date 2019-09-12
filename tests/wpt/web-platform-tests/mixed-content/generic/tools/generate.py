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
        self.selection_pattern = \
              '%(source_context_list)s.%(delivery_type)s/' + \
              '%(delivery_value)s/' + \
              '%(subresource)s/' + \
              '%(origin)s.%(redirection)s.%(source_scheme)s'

        self.test_file_path_pattern = 'gen/' + self.selection_pattern + '.html'

        self.test_description_template = 'Mixed-Content: Expects %(expectation)s for %(subresource)s to %(origin)s origin and %(redirection)s redirection from %(source_scheme)s context.'

        self.test_page_title_template = 'Mixed-Content: %s'

        self.helper_js = '/mixed-content/generic/test-case.sub.js'

        # For debug target only.
        self.sanity_checker_js = '/mixed-content/generic/sanity-checker.js'
        self.spec_json_js = '/mixed-content/spec_json.js'

        self.test_case_name = 'TestCase'

        script_directory = os.path.dirname(os.path.abspath(__file__))
        self.spec_directory = os.path.abspath(
            os.path.join(script_directory, '..', '..'))


if __name__ == '__main__':
    generate.main(MixedContentConfig())
