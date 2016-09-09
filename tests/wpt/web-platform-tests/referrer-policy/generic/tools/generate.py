#!/usr/bin/env python

import os, sys, json
from common_paths import *
import spec_validator
import argparse


def expand_test_expansion_pattern(spec_test_expansion, test_expansion_schema):
    expansion = {}
    for artifact in spec_test_expansion:
        artifact_value = spec_test_expansion[artifact]
        if artifact_value == '*':
            expansion[artifact] = test_expansion_schema[artifact]
        elif isinstance(artifact_value, list):
            expansion[artifact] = artifact_value
        else:
            expansion[artifact] = [artifact_value]

    return expansion


def permute_expansion(expansion, selection = {}, artifact_index = 0):
    artifact_order = ['delivery_method', 'redirection', 'origin',
                      'source_protocol', 'target_protocol', 'subresource',
                      'referrer_url', 'name']

    if artifact_index >= len(artifact_order):
        yield selection
        return

    artifact_key = artifact_order[artifact_index]

    for artifact_value in expansion[artifact_key]:
        selection[artifact_key] = artifact_value
        for next_selection in permute_expansion(expansion,
                                                selection,
                                                artifact_index + 1):
            yield next_selection


def generate_selection(selection, spec, subresource_path,
                       test_html_template_basename):
    selection['spec_name'] = spec['name']
    selection['spec_title'] = spec['title']
    selection['spec_description'] = spec['description']
    selection['spec_specification_url'] = spec['specification_url']
    selection['subresource_path'] = subresource_path
    # Oddball: it can be None, so in JS it's null.
    selection['referrer_policy_json'] = json.dumps(spec['referrer_policy'])

    test_filename = test_file_path_pattern % selection
    test_directory = os.path.dirname(test_filename)
    full_path = os.path.join(spec_directory, test_directory)

    test_html_template = get_template(test_html_template_basename)
    test_js_template = get_template("test.js.template")
    disclaimer_template = get_template('disclaimer.template')
    test_description_template = get_template("test_description.template")

    html_template_filename = os.path.join(template_directory,
                                          test_html_template_basename)
    generated_disclaimer = disclaimer_template \
        % {'generating_script_filename': os.path.relpath(__file__,
                                                         test_root_directory),
           'html_template_filename': os.path.relpath(html_template_filename,
                                                     test_root_directory)}

    # Adjust the template for the test invoking JS. Indent it to look nice.
    selection['generated_disclaimer'] = generated_disclaimer.rstrip()
    test_description_template = \
        test_description_template.rstrip().replace("\n", "\n" + " " * 33)
    selection['test_description'] = test_description_template % selection

    # Adjust the template for the test invoking JS. Indent it to look nice.
    indent = "\n" + " " * 6;
    test_js_template = indent + test_js_template.replace("\n", indent);
    selection['test_js'] = test_js_template % selection

    # Directory for the test files.
    try:
        os.makedirs(full_path)
    except:
        pass

    selection['meta_delivery_method'] = ''

    if spec['referrer_policy'] != None:
        if selection['delivery_method'] == 'meta-referrer':
            selection['meta_delivery_method'] = \
                '<meta name="referrer" content="%(referrer_policy)s">' % spec
        elif selection['delivery_method'] == 'http-rp':
            selection['meta_delivery_method'] = \
                "<!-- No meta: Referrer policy delivered via HTTP headers. -->"
            test_headers_filename = test_filename + ".headers"
            with open(test_headers_filename, "w") as f:
                f.write('Referrer-Policy: ' + \
                        '%(referrer_policy)s\n' % spec)
                # TODO(kristijanburnik): Limit to WPT origins.
                f.write('Access-Control-Allow-Origin: *\n')
        elif selection['delivery_method'] == 'attr-referrer':
            # attr-referrer is supported by the JS test wrapper.
            pass
        elif selection['delivery_method'] == 'rel-noreferrer':
            # rel=noreferrer is supported by the JS test wrapper.
            pass
        else:
            raise ValueError('Not implemented delivery_method: ' \
                              + selection['delivery_method'])

    # Obey the lint and pretty format.
    if len(selection['meta_delivery_method']) > 0:
        selection['meta_delivery_method'] = "\n    " + \
                                            selection['meta_delivery_method']

    with open(test_filename, 'w') as f:
        f.write(test_html_template % selection)


def generate_test_source_files(spec_json, target):
    test_expansion_schema = spec_json['test_expansion_schema']
    specification = spec_json['specification']

    spec_json_js_template = get_template('spec_json.js.template')
    with open(generated_spec_json_filename, 'w') as f:
        f.write(spec_json_js_template
                % {'spec_json': json.dumps(spec_json)})

    # Choose a debug/release template depending on the target.
    html_template = "test.%s.html.template" % target

    # Create list of excluded tests.
    exclusion_dict = {}
    for excluded_pattern in spec_json['excluded_tests']:
        excluded_expansion = \
            expand_test_expansion_pattern(excluded_pattern,
                                          test_expansion_schema)
        for excluded_selection in permute_expansion(excluded_expansion):
            excluded_selection_path = selection_pattern % excluded_selection
            exclusion_dict[excluded_selection_path] = True

    for spec in specification:
        for spec_test_expansion in spec['test_expansion']:
            expansion = expand_test_expansion_pattern(spec_test_expansion,
                                                      test_expansion_schema)
            for selection in permute_expansion(expansion):
                selection_path = selection_pattern % selection
                if not selection_path in exclusion_dict:
                    subresource_path = \
                        spec_json["subresource_path"][selection["subresource"]]
                    generate_selection(selection,
                                       spec,
                                       subresource_path,
                                       html_template)
                else:
                    print 'Excluding selection:', selection_path


def main(target):
    spec_json = load_spec_json();
    spec_validator.assert_valid_spec_json(spec_json)
    generate_test_source_files(spec_json, target)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description='Test suite generator utility')
    parser.add_argument('-t', '--target', type = str,
        choices = ("release", "debug"), default = "release",
        help = 'Sets the appropriate template for generating tests')
    # TODO(kristijanburnik): Add option for the spec_json file.
    args = parser.parse_args()
    main(args.target)
