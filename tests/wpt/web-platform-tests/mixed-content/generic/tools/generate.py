#!/usr/bin/env python

import os
import sys

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), '..', '..', '..', 'common', 'security-features', 'tools'))
import generate


class MixedContentConfig(object):
  def __init__(self):
    self.selection_pattern = '%(subresource)s/' + \
                             '%(opt_in_method)s/' + \
                             '%(origin)s/' + \
                             '%(context_nesting)s/' + \
                             '%(redirection)s/'

    self.test_file_path_pattern = self.selection_pattern + \
                                  '%(spec_name)s/' + \
                                  '%(name)s.%(source_scheme)s.html'

    self.test_description_template = '''opt_in_method: %(opt_in_method)s
origin: %(origin)s
source_scheme: %(source_scheme)s
context_nesting: %(context_nesting)s
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
    self.spec_directory = os.path.abspath(os.path.join(script_directory, '..', '..'))

  def handleDelivery(self, selection, spec):
    opt_in_method = selection['opt_in_method']

    meta = ''
    headers = []

    # TODO(kristijanburnik): Implement the opt-in-method here.
    if opt_in_method == 'meta-csp':
        meta = '<meta http-equiv="Content-Security-Policy" ' + \
               'content="block-all-mixed-content">'
    elif opt_in_method == 'http-csp':
        headers.append("Content-Security-Policy: block-all-mixed-content")
    elif opt_in_method == 'no-opt-in':
        pass
    else:
        raise ValueError("Invalid opt_in_method %s" % opt_in_method)

    return {"meta": meta, "headers": headers}


if __name__ == '__main__':
    generate.main(MixedContentConfig())
