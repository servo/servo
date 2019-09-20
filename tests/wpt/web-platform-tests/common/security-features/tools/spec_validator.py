#!/usr/bin/env python

from __future__ import print_function

import json, sys


def assert_non_empty_string(obj, field):
    assert field in obj, 'Missing field "%s"' % field
    assert isinstance(obj[field], basestring), \
        'Field "%s" must be a string' % field
    assert len(obj[field]) > 0, 'Field "%s" must not be empty' % field


def assert_non_empty_list(obj, field):
    assert isinstance(obj[field], list), \
        '%s must be a list' % field
    assert len(obj[field]) > 0, \
        '%s list must not be empty' % field


def assert_non_empty_dict(obj, field):
    assert isinstance(obj[field], dict), \
        '%s must be a dict' % field
    assert len(obj[field]) > 0, \
        '%s dict must not be empty' % field


def assert_contains(obj, field):
    assert field in obj, 'Must contain field "%s"' % field


def assert_value_from(obj, field, items):
    assert obj[field] in items, \
         'Field "%s" must be from: %s' % (field, str(items))


def assert_atom_or_list_items_from(obj, field, items):
    if isinstance(obj[field], basestring) or isinstance(
            obj[field], int) or obj[field] is None:
        assert_value_from(obj, field, items)
        return

    assert isinstance(obj[field], list), '%s must be a list' % field
    for allowed_value in obj[field]:
        assert allowed_value != '*', "Wildcard is not supported for lists!"
        assert allowed_value in items, \
            'Field "%s" must be from: %s' % (field, str(items))


def assert_contains_only_fields(obj, expected_fields):
    for expected_field in expected_fields:
        assert_contains(obj, expected_field)

    for actual_field in obj:
        assert actual_field in expected_fields, \
                'Unexpected field "%s".' % actual_field


def leaf_values(schema):
    if isinstance(schema, list):
        return schema
    ret = []
    for _, sub_schema in schema.iteritems():
        ret += leaf_values(sub_schema)
    return ret


def assert_value_unique_in(value, used_values):
    assert value not in used_values, 'Duplicate value "%s"!' % str(value)
    used_values[value] = True


def assert_valid_artifact(exp_pattern, artifact_key, schema):
    if isinstance(schema, list):
        assert_atom_or_list_items_from(exp_pattern, artifact_key,
                                       ["*"] + schema)
        return

    for sub_artifact_key, sub_schema in schema.iteritems():
        assert_valid_artifact(exp_pattern[artifact_key], sub_artifact_key,
                              sub_schema)


def validate(spec_json, details):
    """ Validates the json specification for generating tests. """

    details['object'] = spec_json
    assert_contains_only_fields(spec_json, [
        "specification", "delivery_key", "subresource_schema",
        "source_context_schema", "source_context_list_schema",
        "test_expansion_schema", "excluded_tests"
    ])
    assert_non_empty_list(spec_json, "specification")
    assert_non_empty_dict(spec_json, "test_expansion_schema")
    assert_non_empty_list(spec_json, "excluded_tests")

    specification = spec_json['specification']
    test_expansion_schema = spec_json['test_expansion_schema']
    excluded_tests = spec_json['excluded_tests']

    valid_test_expansion_fields = ['name'] + test_expansion_schema.keys()

    # Should be consistent with `sourceContextMap` in
    # `/common/security-features/resources/common.sub.js`.
    valid_source_context_names = [
        "top", "iframe", "iframe-blank", "srcdoc", "worker-classic",
        "worker-module", "worker-classic-data", "worker-module-data"
    ]

    valid_subresource_names = [
        "a-tag", "area-tag", "audio-tag", "form-tag", "iframe-tag", "img-tag",
        "link-css-tag", "link-prefetch-tag", "object-tag", "picture-tag",
        "script-tag", "video-tag"
    ] + ["beacon", "fetch", "xhr", "websocket"] + [
        "worker-classic", "worker-module", "worker-import",
        "worker-import-data", "sharedworker-classic", "sharedworker-module",
        "sharedworker-import", "sharedworker-import-data",
        "serviceworker-classic", "serviceworker-module",
        "serviceworker-import", "serviceworker-import-data"
    ] + [
        "worklet-animation", "worklet-audio", "worklet-layout",
        "worklet-paint", "worklet-animation-import", "worklet-audio-import",
        "worklet-layout-import", "worklet-paint-import",
        "worklet-animation-import-data", "worklet-audio-import-data",
        "worklet-layout-import-data", "worklet-paint-import-data"
    ]

    # Validate each single spec.
    for spec in specification:
        details['object'] = spec

        # Validate required fields for a single spec.
        assert_contains_only_fields(spec, [
            'name', 'title', 'description', 'specification_url',
            'test_expansion'
        ])
        assert_non_empty_string(spec, 'name')
        assert_non_empty_string(spec, 'title')
        assert_non_empty_string(spec, 'description')
        assert_non_empty_string(spec, 'specification_url')
        assert_non_empty_list(spec, 'test_expansion')

        # Validate spec's test expansion.
        used_spec_names = {}

        for spec_exp in spec['test_expansion']:
            details['object'] = spec_exp
            assert_non_empty_string(spec_exp, 'name')
            # The name is unique in same expansion group.
            assert_value_unique_in((spec_exp['expansion'], spec_exp['name']),
                                   used_spec_names)
            assert_contains_only_fields(spec_exp, valid_test_expansion_fields)

            for artifact in test_expansion_schema:
                details['test_expansion_field'] = artifact
                assert_valid_artifact(spec_exp, artifact,
                                      test_expansion_schema[artifact])
                del details['test_expansion_field']

    # Validate source_context_schema.
    details['object'] = spec_json['source_context_schema']
    assert_contains_only_fields(
        spec_json['source_context_schema'],
        ['supported_delivery_type', 'supported_subresource'])
    assert_contains_only_fields(
        spec_json['source_context_schema']['supported_delivery_type'],
        valid_source_context_names)
    for source_context in spec_json['source_context_schema'][
            'supported_delivery_type']:
        assert_valid_artifact(
            spec_json['source_context_schema']['supported_delivery_type'],
            source_context, test_expansion_schema['delivery_type'])
    assert_contains_only_fields(
        spec_json['source_context_schema']['supported_subresource'],
        valid_source_context_names)
    for source_context in spec_json['source_context_schema'][
            'supported_subresource']:
        assert_valid_artifact(
            spec_json['source_context_schema']['supported_subresource'],
            source_context, leaf_values(test_expansion_schema['subresource']))

    # Validate subresource_schema.
    details['object'] = spec_json['subresource_schema']
    assert_contains_only_fields(spec_json['subresource_schema'],
                                ['supported_delivery_type'])
    assert_contains_only_fields(
        spec_json['subresource_schema']['supported_delivery_type'],
        leaf_values(test_expansion_schema['subresource']))
    for subresource in spec_json['subresource_schema'][
            'supported_delivery_type']:
        assert_valid_artifact(
            spec_json['subresource_schema']['supported_delivery_type'],
            subresource, test_expansion_schema['delivery_type'])

    # Validate the test_expansion schema members.
    details['object'] = test_expansion_schema
    assert_contains_only_fields(test_expansion_schema, [
        'expansion', 'source_scheme', 'source_context_list', 'delivery_type',
        'delivery_value', 'redirection', 'subresource', 'origin', 'expectation'
    ])
    assert_atom_or_list_items_from(test_expansion_schema, 'expansion',
                                   ['default', 'override'])
    assert_atom_or_list_items_from(test_expansion_schema, 'source_scheme',
                                   ['http', 'https'])
    assert_atom_or_list_items_from(
        test_expansion_schema, 'source_context_list',
        spec_json['source_context_list_schema'].keys())

    # Should be consistent with `preprocess_redirection` in
    # `/common/security-features/subresource/subresource.py`.
    assert_atom_or_list_items_from(test_expansion_schema, 'redirection', [
        'no-redirect', 'keep-origin', 'swap-origin', 'keep-scheme',
        'swap-scheme', 'downgrade'
    ])
    for subresource in leaf_values(test_expansion_schema['subresource']):
        assert subresource in valid_subresource_names, "Invalid subresource %s" % subresource
    # Should be consistent with getSubresourceOrigin() in
    # `/common/security-features/resources/common.sub.js`.
    assert_atom_or_list_items_from(test_expansion_schema, 'origin', [
        'same-http', 'same-https', 'same-ws', 'same-wss', 'cross-http',
        'cross-https', 'cross-ws', 'cross-wss', 'same-http-downgrade',
        'cross-http-downgrade', 'same-ws-downgrade', 'cross-ws-downgrade'
    ])

    # Validate excluded tests.
    details['object'] = excluded_tests
    for excluded_test_expansion in excluded_tests:
        assert_contains_only_fields(excluded_test_expansion,
                                    valid_test_expansion_fields)
        details['object'] = excluded_test_expansion
        for artifact in test_expansion_schema:
            details['test_expansion_field'] = artifact
            assert_valid_artifact(excluded_test_expansion, artifact,
                                  test_expansion_schema[artifact])
            del details['test_expansion_field']

    del details['object']


def assert_valid_spec_json(spec_json):
    error_details = {}
    try:
        validate(spec_json, error_details)
    except AssertionError as err:
        print('ERROR:', err.message)
        print(json.dumps(error_details, indent=4))
        sys.exit(1)


def main():
    spec_json = load_spec_json()
    assert_valid_spec_json(spec_json)
    print("Spec JSON is valid.")


if __name__ == '__main__':
    main()
