# Copyright 2016 The Chromium Authors. All rights reserved.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.
#
# TODO(509038): Delete the file in LayoutTests/bluetooth after all the script
# tests have been migrated to this directory.
"""Generator script for Web Bluetooth LayoutTests.

For each script-tests/X.js creates the following test files depending on the
contents of X.js
- getPrimaryService/X.https.window.js
- getPrimaryServices/X.https.window.js
- getPrimaryServices/X-with-uuid.https.window.js

script-tests/X.js files should contain "CALLS([variation1 | variation2 | ...])"
tokens that indicate what files to generate. Each variation in CALLS([...])
should corresponds to a js function call and its arguments. Additionally a
variation can end in [UUID] to indicate that the generated file's name should
have the -with-uuid suffix.

The PREVIOUS_CALL token will be replaced with the function that replaced CALLS.

The FUNCTION_NAME token will be replaced with the name of the function that
replaced CALLS.

For example, for the following template file:

// script-tests/example.js
promise_test(() => {
    return navigator.bluetooth.requestDevice(...)
        .then(device => device.gatt.CALLS([
            getPrimaryService('heart_rate')|
            getPrimaryServices('heart_rate')[UUID]]))
        .then(device => device.gatt.PREVIOUS_CALL);
}, 'example test for FUNCTION_NAME');

this script will generate:

// getPrimaryService/example.https.window.js
promise_test(() => {
    return navigator.bluetooth.requestDevice(...)
        .then(device => device.gatt.getPrimaryService('heart_rate'))
        .then(device => device.gatt.getPrimaryService('heart_rate'));
}, 'example test for getPrimaryService');

// getPrimaryServices/example-with-uuid.https.window.js
promise_test(() => {
    return navigator.bluetooth.requestDevice(...)
        .then(device => device.gatt.getPrimaryServices('heart_rate'))
        .then(device => device.gatt.getPrimaryServices('heart_rate'));
}, 'example test for getPrimaryServices');

Run
$ python //third_party/WebKit/LayoutTests/bluetooth/generate.py
and commit the generated files.
"""

import fnmatch
import os
import re
import sys
import logging

TEMPLATES_DIR = 'script-tests'


class GeneratedTest:

    def __init__(self, data, path, template):
        self.data = data
        self.path = path
        self.template = template


def GetGeneratedTests():
    """Yields a GeneratedTest for each call in templates in script-tests."""
    bluetooth_tests_dir = os.path.dirname(os.path.realpath(__file__))

    # Read Base Test Template.
    base_template_file_handle = open(
        os.path.join(
            bluetooth_tests_dir,
            TEMPLATES_DIR,
            'base_test_js.template'
        ), 'r')
    base_template_file_data = base_template_file_handle.read().decode('utf-8')
    base_template_file_handle.close()

    # Get Templates.

    template_path = os.path.join(bluetooth_tests_dir, TEMPLATES_DIR)

    available_templates = []
    for root, _, files in os.walk(template_path):
        for template in files:
            if template.endswith('.js'):
                available_templates.append(os.path.join(root, template))

    # Generate Test Files
    for template in available_templates:
        # Read template
        template_file_handle = open(template, 'r')
        template_file_data = template_file_handle.read().decode('utf-8')
        template_file_handle.close()

        template_name = os.path.splitext(os.path.basename(template))[0]

        # Find function names in multiline pattern: CALLS( [ function_name,function_name2[UUID] ])
        result = re.search(
            r'CALLS\(' + # CALLS(
            r'[^\[]*' +  # Any characters not [, allowing for new lines.
            r'\[' +      # [
            r'(.*?)' +   # group matching: function_name(), function_name2[UUID]
            r'\]\)',     # adjacent closing characters: ])
            template_file_data, re.MULTILINE | re.DOTALL)

        if result is None:
            raise Exception('Template must contain \'CALLS\' tokens')

        new_test_file_data = base_template_file_data.replace('TEST',
            template_file_data)
        # Replace CALLS([...]) with CALLS so that we don't have to replace the
        # CALLS([...]) for every new test file.
        new_test_file_data = new_test_file_data.replace(result.group(), 'CALLS')

        # Replace 'PREVIOUS_CALL' with 'CALLS' so that we can replace it while
        # replacing CALLS.
        new_test_file_data = new_test_file_data.replace('PREVIOUS_CALL', 'CALLS')

        for call in result.group(1).split('|'):
            # Parse call
            call = call.strip()
            function_name, args, uuid_suffix = re.search(r'(.*?)\((.*)\)(\[UUID\])?', call).groups()

            # Replace template tokens
            call_test_file_data = new_test_file_data
            call_test_file_data = call_test_file_data.replace('CALLS', '{}({})'.format(function_name, args))
            call_test_file_data = call_test_file_data.replace('FUNCTION_NAME', function_name)

            # Get test file name
            group_dir = os.path.basename(os.path.abspath(os.path.join(template, os.pardir)))

            call_test_file_name = 'gen-{}{}.https.window.js'.format(template_name, '-with-uuid' if uuid_suffix else '')
            call_test_file_path = os.path.join(bluetooth_tests_dir, group_dir, function_name, call_test_file_name)

            yield GeneratedTest(call_test_file_data, call_test_file_path, template)

def main():
    logging.basicConfig(level=logging.INFO)
    previous_generated_files = set()
    current_path = os.path.dirname(os.path.realpath(__file__))
    for root, _, filenames in os.walk(current_path):
        for filename in fnmatch.filter(filenames, 'gen-*.https.window.js'):
            previous_generated_files.add(os.path.join(root, filename))

    generated_files = set()
    for generated_test in GetGeneratedTests():
        prev_len = len(generated_files)
        generated_files.add(generated_test.path)
        if prev_len == len(generated_files):
            logging.info('Generated the same test twice for template:\n%s',
                       generated_test.template)

        # Create or open test file
        directory = os.path.dirname(generated_test.path)
        if not os.path.exists(directory):
            os.makedirs(directory)
        test_file_handle = open(generated_test.path, 'wb')

        # Write contents
        test_file_handle.write(generated_test.data.encode('utf-8'))
        test_file_handle.close()

    new_generated_files = generated_files - previous_generated_files
    if len(new_generated_files) != 0:
        logging.info('Newly generated tests:')
        for generated_file in new_generated_files:
              logging.info(generated_file)

    obsolete_files = previous_generated_files - generated_files
    if len(obsolete_files) != 0:
        logging.warning('The following files might be obsolete:')
        for generated_file in obsolete_files:
            logging.warning(generated_file)



if __name__ == '__main__':
    sys.exit(main())
