from __future__ import print_function

import argparse
import copy
import json
import os
import sys

import spec_validator
import util


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


def permute_expansion(expansion,
                      artifact_order,
                      selection={},
                      artifact_index=0):
    assert isinstance(artifact_order, list), "artifact_order should be a list"

    if artifact_index >= len(artifact_order):
        yield selection
        return

    artifact_key = artifact_order[artifact_index]

    for artifact_value in expansion[artifact_key]:
        selection[artifact_key] = artifact_value
        for next_selection in permute_expansion(expansion, artifact_order,
                                                selection, artifact_index + 1):
            yield next_selection


# Dumps the test config `selection` into a serialized JSON string.
# We omit `name` parameter because it is not used by tests.
def dump_test_parameters(selection):
    selection = dict(selection)
    del selection['name']

    return json.dumps(
        selection,
        indent=2,
        separators=(',', ': '),
        sort_keys=True,
        cls=util.CustomEncoder)


def get_test_filename(config, selection):
    '''Returns the filname for the main test HTML file'''

    selection_for_filename = copy.deepcopy(selection)
    # Use 'unset' rather than 'None' in test filenames.
    if selection_for_filename['delivery_value'] is None:
        selection_for_filename['delivery_value'] = 'unset'

    return os.path.join(config.spec_directory,
                        config.test_file_path_pattern % selection_for_filename)


def handle_deliveries(policy_deliveries):
    '''
    Generate <meta> elements and HTTP headers for the given list of
    PolicyDelivery.
    TODO(hiroshige): Merge duplicated code here, scope/document.py, etc.
    '''

    meta = ''
    headers = {}

    for delivery in policy_deliveries:
        if delivery.value is None:
            continue
        if delivery.key == 'referrerPolicy':
            if delivery.delivery_type == 'meta':
                meta += \
                    '<meta name="referrer" content="%s">' % delivery.value
            elif delivery.delivery_type == 'http-rp':
                headers['Referrer-Policy'] = delivery.value
                # TODO(kristijanburnik): Limit to WPT origins.
                headers['Access-Control-Allow-Origin'] = '*'
            else:
                raise Exception(
                    'Invalid delivery_type: %s' % delivery.delivery_type)
        elif delivery.key == 'mixedContent':
            assert (delivery.value == 'opt-in')
            if delivery.delivery_type == 'meta':
                meta += '<meta http-equiv="Content-Security-Policy" ' + \
                       'content="block-all-mixed-content">'
            elif delivery.delivery_type == 'http-rp':
                headers['Content-Security-Policy'] = 'block-all-mixed-content'
            else:
                raise Exception(
                    'Invalid delivery_type: %s' % delivery.delivery_type)
        else:
            raise Exception('Invalid delivery_key: %s' % delivery.key)
    return {"meta": meta, "headers": headers}


def generate_selection(spec_json, config, selection, spec,
                       test_html_template_basename):
    test_filename = get_test_filename(config, selection)

    target_policy_delivery = util.PolicyDelivery(selection['delivery_type'],
                                                 selection['delivery_key'],
                                                 selection['delivery_value'])
    del selection['delivery_type']
    del selection['delivery_key']
    del selection['delivery_value']

    # Parse source context list and policy deliveries of source contexts.
    # `util.ShouldSkip()` exceptions are raised if e.g. unsuppported
    # combinations of source contexts and policy deliveries are used.
    source_context_list_scheme = spec_json['source_context_list_schema'][
        selection['source_context_list']]
    selection['source_context_list'] = [
        util.SourceContext.from_json(source_context, target_policy_delivery,
                                     spec_json['source_context_schema'])
        for source_context in source_context_list_scheme['sourceContextList']
    ]

    # Check if the subresource is supported by the innermost source context.
    innermost_source_context = selection['source_context_list'][-1]
    supported_subresource = spec_json['source_context_schema'][
        'supported_subresource'][innermost_source_context.source_context_type]
    if supported_subresource != '*':
        if selection['subresource'] not in supported_subresource:
            raise util.ShouldSkip()

    # Parse subresource policy deliveries.
    selection[
        'subresource_policy_deliveries'] = util.PolicyDelivery.list_from_json(
            source_context_list_scheme['subresourcePolicyDeliveries'],
            target_policy_delivery, spec_json['subresource_schema']
            ['supported_delivery_type'][selection['subresource']])

    # We process the top source context below, and do not include it in
    # `test_parameters` in JavaScript.
    top_source_context = selection['source_context_list'].pop(0)
    assert (top_source_context.source_context_type == 'top')

    test_parameters = dump_test_parameters(selection)
    # Adjust the template for the test invoking JS. Indent it to look nice.
    indent = "\n" + " " * 8
    test_parameters = test_parameters.replace("\n", indent)

    selection['test_js'] = '''
      %s(
        %s,
        document.querySelector("meta[name=assert]").content,
        new SanityChecker()
      ).start();
      ''' % (config.test_case_name, test_parameters)

    selection['spec_name'] = spec['name']
    selection[
        'test_page_title'] = config.test_page_title_template % spec['title']
    selection['spec_description'] = spec['description']
    selection['spec_specification_url'] = spec['specification_url']
    selection['helper_js'] = config.helper_js
    selection['sanity_checker_js'] = config.sanity_checker_js
    selection['spec_json_js'] = config.spec_json_js

    test_headers_filename = test_filename + ".headers"
    test_directory = os.path.dirname(test_filename)

    test_html_template = util.get_template(test_html_template_basename)
    disclaimer_template = util.get_template('disclaimer.template')

    html_template_filename = os.path.join(util.template_directory,
                                          test_html_template_basename)
    generated_disclaimer = disclaimer_template \
        % {'generating_script_filename': os.path.relpath(sys.argv[0],
                                                         util.test_root_directory),
           'html_template_filename': os.path.relpath(html_template_filename,
                                                     util.test_root_directory)}

    # Adjust the template for the test invoking JS. Indent it to look nice.
    selection['generated_disclaimer'] = generated_disclaimer.rstrip()
    selection[
        'test_description'] = config.test_description_template % selection
    selection['test_description'] = \
        selection['test_description'].rstrip().replace("\n", "\n" + " " * 33)

    # Directory for the test files.
    try:
        os.makedirs(test_directory)
    except:
        pass

    delivery = handle_deliveries(top_source_context.policy_deliveries)

    if len(delivery['headers']) > 0:
        with open(test_headers_filename, "w") as f:
            for header in delivery['headers']:
                f.write('%s: %s\n' % (header, delivery['headers'][header]))

    selection['meta_delivery_method'] = delivery['meta']
    # Obey the lint and pretty format.
    if len(selection['meta_delivery_method']) > 0:
        selection['meta_delivery_method'] = "\n    " + \
                                            selection['meta_delivery_method']

    # Write out the generated HTML file.
    util.write_file(test_filename, test_html_template % selection)


def generate_test_source_files(config, spec_json, target):
    test_expansion_schema = spec_json['test_expansion_schema']
    specification = spec_json['specification']

    spec_json_js_template = util.get_template('spec_json.js.template')
    generated_spec_json_filename = os.path.join(config.spec_directory,
                                                "spec_json.js")
    util.write_file(
        generated_spec_json_filename,
        spec_json_js_template % {'spec_json': json.dumps(spec_json)})

    # Choose a debug/release template depending on the target.
    html_template = "test.%s.html.template" % target

    artifact_order = test_expansion_schema.keys() + ['name']
    artifact_order.remove('expansion')

    # Create list of excluded tests.
    exclusion_dict = {}
    for excluded_pattern in spec_json['excluded_tests']:
        excluded_expansion = \
            expand_pattern(excluded_pattern, test_expansion_schema)
        for excluded_selection in permute_expansion(excluded_expansion,
                                                    artifact_order):
            excluded_selection_path = config.selection_pattern % excluded_selection
            exclusion_dict[excluded_selection_path] = True

    for spec in specification:
        # Used to make entries with expansion="override" override preceding
        # entries with the same |selection_path|.
        output_dict = {}

        for expansion_pattern in spec['test_expansion']:
            expansion = expand_pattern(expansion_pattern,
                                       test_expansion_schema)
            for selection in permute_expansion(expansion, artifact_order):
                selection['delivery_key'] = spec_json['delivery_key']
                selection_path = config.selection_pattern % selection
                if not selection_path in exclusion_dict:
                    if selection_path in output_dict:
                        if expansion_pattern['expansion'] != 'override':
                            print(
                                "Error: %s's expansion is default but overrides %s"
                                % (selection['name'],
                                   output_dict[selection_path]['name']))
                            sys.exit(1)
                    output_dict[selection_path] = copy.deepcopy(selection)
                else:
                    print('Excluding selection:', selection_path)

        for selection_path in output_dict:
            selection = output_dict[selection_path]
            try:
                generate_selection(spec_json, config, selection, spec,
                                   html_template)
            except util.ShouldSkip:
                continue


def main(config):
    parser = argparse.ArgumentParser(
        description='Test suite generator utility')
    parser.add_argument(
        '-t',
        '--target',
        type=str,
        choices=("release", "debug"),
        default="release",
        help='Sets the appropriate template for generating tests')
    parser.add_argument(
        '-s',
        '--spec',
        type=str,
        default=None,
        help='Specify a file used for describing and generating the tests')
    # TODO(kristijanburnik): Add option for the spec_json file.
    args = parser.parse_args()

    if args.spec:
        config.spec_directory = args.spec

    spec_filename = os.path.join(config.spec_directory, "spec.src.json")
    spec_json = util.load_spec_json(spec_filename)
    spec_validator.assert_valid_spec_json(spec_json)

    generate_test_source_files(config, spec_json, args.target)
