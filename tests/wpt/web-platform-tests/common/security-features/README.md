

This directory contains the common infrastructure for the following tests (also referred below as projects).

- referrer-policy/
- mixed-content/
- upgrade-insecure-requests/

Subdirectories:

- `resources`:
    Serves JavaScript test helpers.
- `subresource`:
    Serves subresources, with support for redirects, stash, etc.
    The subresource paths are managed by `subresourceMap` and
    fetched in `requestVia*()` functions in `resources/common.js`.
- `scope`:
    Serves nested contexts, such as iframe documents or workers.
    Used from `invokeFrom*()` functions in `resources/common.js`.
- `tools`:
    Scripts that generate test HTML files. Not used while running tests.
- `/referrer-policy/generic/subresource-test`:
    Sanity checking tests for subresource invocation
    (This is still placed outside common/)

# Test generator

The test generator (`common/security-features/tools`) generates test HTML files from templates and a seed (`spec.src.json`) that defines all the test scenarios.

The project (i.e. a WPT subdirectory, for example `referrer-policy/`) that uses the generator should define per-project data and invoke the common generator logic in `common/security-features/tools`.

This is the overview of the project structure:

```
common/security-features/
└── tools/ - the common test generator logic
    └── template/ - the test files templates
project-directory/ (e.g. referrer-policy/)
├── spec.src.json
├── generic/
│   ├── test-case.sub.js - Per-project test helper
│   └── tools/
│       └── generator.py - Per-project generator script
└── gen/ - generated tests
```

Invoking `project-directory/generic/tools/generate.py` will parse the spec JSON and determine which tests to generate (or skip) while using templates.

## Generating the tests

The repository already contains generated tests, so if you're making changes, see the [Removing all generated tests](#removing-all-generated-tests) section below, on how to remove them before you start generating tests which include your changes.

```bash
# Chdir into the project directory.
cd ~/web-platform-tests/project-directory

# Generate the test files under gen/ (HTMLs and .headers files).
./generic/tools/generate.py

# Add all generated tests to the repo.
git add gen/ && git commit -m "Add generated tests"
```

During the generation, the spec is validated by ```common/security-features/tools/spec_validator.py```. This is specially important when you're making changes to  `spec.src.json`. Make sure it's a valid JSON (no comments or trailing commas). The validator reports specific errors (missing keys etc.), if any.

### Removing all generated tests

Simply remove all files under `project-directory/gen/`.

```bash
# Chdir into the project directory.
cd ~/web-platform-tests/project-directory

# Remove all generated tests.
rm -r gen/
```

### Options for generating tests

Note: this section is currently obsolete. Only the release template is working.

The generator script ```./generic/tools/generate.py``` has two targets: ```release``` and ```debug```.

* Using **release** for the target will produce tests using a template for optimizing size and performance. The release template is intended for the official web-platform-tests and possibly other test suites. No sanity checking is done in release mode. Use this option whenever you're checking into web-platform-tests.

* When generating for ```debug```, the produced tests will contain more verbosity and sanity checks. Use this target to identify problems with the test suites when making changes locally. Make sure you don't check in tests generated with the debug target.

Note that **release** is the default target when invoking ```generate.py```.


## Updating the tests

The main test logic lives in ```project-directory/generic/test-case.sub.js``` with helper functions defined in ```/common/security-features/resources/common.js``` so you should probably start there.

For updating the test suites you will most likely do **a subset** of the following:

* Add a new subresource type:

  * Add a new sub-resource python script to `/common/security-features/subresource/`.
  * Add a sanity check test for a sub-resource to `referrer-policy/generic/subresource-test/`.
  * Add a new entry to `subresourceMap` in `/common/security-features/resources/common.js`.
  * Add a new entry to `valid_subresource_names` in `/common/security-features/tools/spec_validator.py`.
  * Add a new entry to `subresource_schema` in `spec.src.json`.
  * Update `source_context_schema` to specify in which source context the subresource can be used.

* Add a new subresource redirection type

  * TODO: to be documented. Example: [https://github.com/web-platform-tests/wpt/pull/18939](https://github.com/web-platform-tests/wpt/pull/18939)

* Add a new subresource origin type

  * TODO: to be documented. Example: [https://github.com/web-platform-tests/wpt/pull/18940](https://github.com/web-platform-tests/wpt/pull/18940)

* Add a new source context (e.g. "module sharedworker global scope")

  * TODO: to be documented. Example: [https://github.com/web-platform-tests/wpt/pull/18904](https://github.com/web-platform-tests/wpt/pull/18904)

* Add a new source context list (e.g. "subresource request from a dedicated worker in a `<iframe srcdoc>`")

  * TODO: to be documented.

* Implement new or update existing assertions in ```project-directory/generic/test-case.sub.js```.

* Exclude or add some tests by updating ```spec.src.json``` test expansions.

* Implement a new delivery method.

  * TODO: to be documented. Currently the support for delivery methods are implemented in many places across `common/security-features/`.

* Regenerate the tests and MANIFEST.json


## The spec JSON format

For examples of spec JSON files, see [referrer-policy/spec.src.json](../../referrer-policy/spec.src.json) or  [mixed-content/spec.src.json](../../mixed-content/spec.src.json).

### Main sections

* **`specification`**

  Top level requirements with description fields and a ```test_expansion``` rule.
  This is closely mimicking the [Referrer Policy specification](http://w3c.github.io/webappsec/specs/referrer-policy/) structure.

* **`excluded_tests`**

  List of ```test_expansion``` patterns expanding into selections which get skipped when generating the tests (aka. blacklisting/suppressing)

* **`test_expansion_schema`**

  Provides valid values for each field.
  Each test expansion can only contain fields and values defined by this schema (or `"*"` values that indicate all the valid values defined this schema).

* **`subresource_schema`**

  Provides metadata of subresources, e.g. supported delivery types for each subresource.

* **`source_context_schema`**

  Provides metadata of each single source context, e.g. supported delivery types and subresources that can be sent from the context.

* **`source_context_list_schema`**

  Provides possible nested combinations of source contexts. See [Source Contexts](#source-contexts) section below for details.

### Test Expansion Patterns

Each field in a test expansion can be in one of the following formats:

* Single match: ```"value"```

* Match any of: ```["value1", "value2", ...]```

* Match all: ```"*"```


**NOTE:** An expansion is always constructive (inclusive), there isn't a negation operator for explicit exclusion. Be aware that using an empty list ```[]``` matches (expands into) exactly nothing. Tests which are to be excluded should be defined in the ```excluded_tests``` section instead.

A single test expansion pattern, be it a requirement or a suppressed pattern, gets expanded into a list of **selections** as follows:

* Expand each field's pattern (single, any of, or all) to list of allowed values (defined by the ```test_expansion_schema```)

* Permute - Recursively enumerate all **selections** across all fields

Be aware that if there is more than one pattern expanding into a same selection, the pattern appearing later in the spec JSON will overwrite a previously generated selection. To make sure this is not undetected when generating, set the value of the ```expansion``` field to ```default``` for an expansion appearing earlier and ```override``` for the one appearing later.

A **selection** is a single **test instance** (scenario) with explicit values that defines a single test. The scenario is then evaluated by the ```TestCase``` in JS. For the rest of the arranging part, examine ```/common/security-features/tools/generate.py``` to see how the values for the templates are produced.


Taking the spec JSON, the generator follows this algorithm:

* Expand all ```excluded_tests``` to create a blacklist of selections

* For each specification requirement: Expand the ```test_expansion``` pattern into selections and check each against the blacklist, if not marked as suppresed, generate the test resources for the selection


### Source Contexts

In **`source_context_list_schema`**, we can specify

- source contexts from where subresource requests are sent, and
- how policies are delivered, by source contexts and/or subresource requests.

- `sourceContextList`: an array of `SourceContext` objects, and
- `subresourcePolicyDeliveries`: an array of `PolicyDelivery` objects.

They have the same object format as described in
`common/security-features/resources/common.js` comments, and are directly
serialized to generated HTML files and passed to JavaScript test code,
except that:

- The first entry of `sourceContextList`'s `sourceContextType` should be
  always `top`, which represents the top-level generated test HTML.
  (This entry is omitted in the JSON passed to JavaScript, but
  the policy deliveries specified here are written as e.g.
  `<meta>` elements in the generated test HTML or HTTP headers)
- Instead of `PolicyDelivery` object (in `sourceContextList` or
  `subresourcePolicyDeliveries`), following placeholder strings can be used.

The keys of `source_context_list_schema` can be used as the values of
`source_context_list` fields, to indicate which source context configuration
to be used.

### PolicyDelivery placeholders

Each test contains

- `delivery_key` (derived from the top-level `delivery_key`) and
- `delivery_value`, `delivery_type` (derived from `test_expansion`),

which represents the **target policy delivery**, the policy delivery to be
tested.

The following placeholder strings in `source_context_list_schema` can be used:

- `"policy"`:
    - Replaced with the target policy delivery.
    - Can be used to specify where the target policy delivery should be
      delivered.
- `"policyIfNonNull"`:
    - Replaced with the target policy delivery, only if it has non-null value.
      If the value is null, then the test file is not generated.
- `"anotherPolicy"`:
    - Replaced with a `PolicyDelivery` object that has a different value from
      the target policy delivery.
    - Can be used to specify e.g. a policy that should be overridden by
      the target policy delivery.

For example, when the target policy delivery is
`{deliveryType: "http-rp", key: "referrerPolicy", value: "no-referrer"}`,

```json
"sourceContextList": [
  {
    "sourceContextType": "top",
    "policyDeliveries": [
      "anotherPolicy"
    ]
  },
  {
    "sourceContextType": "classic-worker",
    "policyDeliveries": [
      "policy"
    ]
  }
]
```

is replaced with

```json
"sourceContextList": [
  {
    "sourceContextType": "top",
    "policyDeliveries": [
      {
        "deliveryType": "meta",
        "key": "referrerPolicy",
        "value": "unsafe-url"
      }
    ]
  },
  {
    "sourceContextType": "classic-worker",
    "policyDeliveries": [
      {
        "deliveryType": "http-rp",
        "key": "referrerPolicy",
        "value": "no-referrer"
      }
    ]
  }
]
```

which indicates

- The top-level Document has `<meta name="referrer" content="unsafe-url">`.
- The classic worker is created with
  `Referrer-Policy: no-referrer` HTTP response headers.

### `source_context_schema` and `subresource_schema`

These represent supported delivery types and subresources
for each source context or subresource type. These are used

- To filter out test files for unsupported combinations of delivery types,
  source contexts and subresources.
- To determine what delivery types should be used for `anotherPolicy`
  placeholder.
