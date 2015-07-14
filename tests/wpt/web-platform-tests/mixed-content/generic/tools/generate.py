#!/usr/bin/env python

import os, sys, json
from common_paths import *
import spec_validator
import argparse


def expand_pattern(expansion_pattern, test_expansion_schema):
    expansion = {}
    for artifact_key in expansion_pattern:
        artifact_value = expansion_pattern[artifact_key]
        if artifact_value == '*':
            expansion[artifact_key] = test_expansion_schema[artifact_key]
        elif isinstance(artifact_value, list):
            expansion[artifact_key] = artifact_value
        elif isinstance(artifact_value, dict):
            # Flattened expansion.
            expansion[artifact_key] = []
            values_dict = expand_pattern(artifact_value,
                                         test_expansion_schema[artifact_key])
            for sub_key in values_dict.keys():
                expansion[artifact_key] += values_dict[sub_key]
        else:
            expansion[artifact_key] = [artifact_value]

    return expansion


def permute_expansion(expansion, artifact_order, selection = {}, artifact_index = 0):
    assert isinstance(artifact_order, list), "artifact_order should be a list"

    if artifact_index >= len(artifact_order):
        yield selection
        return

    artifact_key = artifact_order[artifact_index]

    for artifact_value in expansion[artifact_key]:
        selection[artifact_key] = artifact_value
        for next_selection in permute_expansion(expansion,
                                                artifact_order,
                                                selection,
                                                artifact_index + 1):
            yield next_selection


def generate_selection(selection, spec, test_html_template_basename):
    selection['spec_name'] = spec['name']
    selection['spec_title'] = spec['title']
    selection['spec_description'] = spec['description']
    selection['spec_specification_url'] = spec['specification_url']

    test_filename = test_file_path_pattern % selection
    test_headers_filename = test_filename + ".headers"
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

    # TODO(kristijanburnik): Implement the opt-in-method here.
    opt_in_method = selection['opt_in_method']
    selection['meta_opt_in'] = ''
    if opt_in_method == 'meta-csp':
        selection['meta_opt_in'] = '<meta http-equiv="Content-Security-Policy" ' + \
                                   'content="block-all-mixed-content">'
    elif opt_in_method == 'http-csp':
        opt_in_headers = "Content-Security-Policy: block-all-mixed-content\n"
        write_file(test_headers_filename, opt_in_headers)
    elif opt_in_method == 'no-opt-in':
        pass
    else:
        raise ValueError("Invalid opt_in_method %s" % opt_in_method)

    # Write out the generated HTML file.
    write_file(test_filename, test_html_template % selection)

def generate_test_source_files(spec_json, target):
    test_expansion_schema = spec_json['test_expansion_schema']
    specification = spec_json['specification']

    spec_json_js_template = get_template('spec_json.js.template')
    write_file(generated_spec_json_filename,
               spec_json_js_template % {'spec_json': json.dumps(spec_json)})

    # Choose a debug/release template depending on the target.
    html_template = "test.%s.html.template" % target

    artifact_order = test_expansion_schema.keys() + ['name']

    # Create list of excluded tests.
    exclusion_dict = {}
    for excluded_pattern in spec_json['excluded_tests']:
        excluded_expansion = \
            expand_pattern(excluded_pattern,
                                          test_expansion_schema)
        for excluded_selection in permute_expansion(excluded_expansion, artifact_order):
            excluded_selection_path = selection_pattern % excluded_selection
            exclusion_dict[excluded_selection_path] = True

    for spec in specification:
        for expansion_pattern in spec['test_expansion']:
            expansion = expand_pattern(expansion_pattern,
                                                      test_expansion_schema)
            for selection in permute_expansion(expansion, artifact_order):
                selection_path = selection_pattern % selection
                if not selection_path in exclusion_dict:
                    generate_selection(selection,
                                       spec,
                                       html_template)
                else:
                    print 'Excluding selection:', selection_path


def main(target, spec_filename):
    spec_json = load_spec_json(spec_filename);
    spec_validator.assert_valid_spec_json(spec_json)
    generate_test_source_files(spec_json, target)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description='Test suite generator utility')
    parser.add_argument('-t', '--target', type = str,
        choices = ("release", "debug"), default = "release",
        help = 'Sets the appropriate template for generating tests')
    parser.add_argument('-s', '--spec', type = str, default = None,
        help = 'Specify a file used for describing and generating the tests')
    # TODO(kristijanburnik): Add option for the spec_json file.
    args = parser.parse_args()
    main(args.target, args.spec)
