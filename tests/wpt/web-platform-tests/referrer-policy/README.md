# Referrer-Policy Web Platform Tests

The Referrer-Policy tests are designed for testing browser implementations and conformance to the [W3 Referrer-Policy Specification](http://w3c.github.io/webappsec/specs/referrer-policy/).

## Project structure

The project contains tools, templates and a seed (```spec.src.json```) for generating tests. The main assertion logic resides in JS files in the root of the ```./generic/``` directory.

This is the overview of the project structure:

```
.
└── generic
    ├── subresource - documents being served as sub-resources (python scripts)
    ├── subresource-test - sanity checking tests for resource invocation
    ├── template - the test files template used for generating the tests
    └── tools -  for generating and maintaining the test suite
└── (genereated_tests_for_a_specification_1)
└── (genereated_tests_for_a_specification_2)
└── ...
└── (genereated_tests_for_a_specification_N)
```

## The spec JSON

The ```spec.src.json``` defines all the test scenarios for the referrer policy.

Invoking ```./generic/tools/generate.py``` will parse the spec JSON and determine which tests to generate (or skip) while using templates.


The spec can be validated by running ```./generic/tools/spec_validator.py```. This is specially important when you're making changes to  ```spec.src.json```. Make sure it's a valid JSON (no comments or trailing commas). The validator should be informative and very specific on any issues.

For details about the spec JSON, see **Overview of the spec JSON** below.


## Generating and running the tests

The repository already contains generated tests, so if you're making changes,
see the **Removing all generated tests** section below, on how to remove them before you start generating tests which include your changes.

Start from the command line:

```bash

# Chdir into the tests directory.
cd ~/web-platform-tests/referrer-policy

# Generate the test resources.
./generic/tools/generate.py

# Add all generated tests to the repo.
git add * && git commit -m "Add generated tests"

# Regenerate the manifest.
../manifest

```

Navigate to [http://web-platform.test:8000/tools/runner/index.html](http://web-platform.test:8000/tools/runner/index.html).

Run tests under path: ```/referrer-policy```.

Click start.


## Options for generating tests

The generator script ```./generic/tools/generate.py``` has two targets: ```release``` and ```debug```.

* Using **release** for the target will produce tests using a template for optimizing size and performance. The release template is intended for the official web-platform-tests and possibly other test suites. No sanity checking is done in release mode. Use this option whenever you're checking into web-platform-tests.

* When generating for ```debug```, the produced tests will contain more verbosity and sanity checks. Use this target to identify problems with the test suite when making changes locally. Make sure you don't check in tests generated with the debug target.

Note that **release** is the default target when invoking ```generate.py```.


## Removing all generated tests

```bash
# Chdir into the tests directory.
cd ~/web-platform-tests/referrer-policy

# Remove all generated tests.
./generic/tools/clean.py

# Remove all generated tests to the repo.
git add * && git commit -m "Remove generated tests"

# Regenerate the manifest.
../manifest
```

**Important:**
The ```./generic/tools/clean.py``` utility will only work if there is a valid ```spec.src.json``` and previously generated directories match the specification requirement names. So make sure you run ```clean.py``` before you alter the specification section of the spec JSON.


## Updating the tests

The main test logic lives in ```./generic/referrer-policy-test-case.js``` with helper functions defined in ```./generic/common.js``` so you should probably start there.

For updating the test suite you will most likely do **a subset** of the following:

* Add a new sub-resource python script to ```./generic/subresource/```,
  and update the reference to it in ```spec.src.json```.

* Add a sanity check test for a sub-resource to ```./generic/subresource-test/```.

* Implement new or update existing assertions in ```./generic/referrer-policy-test-case.js```.

* Exclude or add some tests by updating ```spec.src.json``` test expansions.

* Update the template files living in ```./generic/template/```.

* Implement a new delivery method via HTTP headers or as part of the test template in  ```./generic/tools/generate.py```

* Update the spec schema by editing ```spec.src.json``` while updating the
  ```./generic/tools/spec_validator.py``` and ```./generic/tools/generate.py```
  and making sure both still work after the change (by running them).

* Regenerate the tests and MANIFEST.json


## Updating the spec and regenerating

When updating the ```spec.src.json```, e.g. by adding a test expansion pattern to the ```excluded_tests``` section or when removing an expansion in the ```specification``` section, make sure to remove all previously generated files which would still get picked up by ```MANIFEST.json``` in the web-platform-tests root. As long as you don't change the specification requirements' names or remove them, you can easily regenerate the tests via command line:

```bash

# Chdir into the tests directory.
cd ~/web-platform-tests/referrer-policy

# Regenerate the test resources.
./generic/tools/regenerate

# Add all the tests to the repo.
git add * && git commit -m "Update generated tests"

# Regenerate the manifest.
../manifest


```


## Overview of the spec JSON

**Main sections:**

* **specification**

  Top level requirements with description fields and a ```test_expansion``` rule.
  This is closely mimicking the [Referrer Policy specification](http://w3c.github.io/webappsec/specs/referrer-policy/) structure.

* **excluded_tests**

  List of ```test_expansion``` patterns expanding into selections which get skipped when generating the tests (aka. blacklisting/suppressing)

* **referrer_policy_schema**

  The schema to validate fields which define the ```referrer_policy``` elsewhere in the JSON.
  A value for a referrer_policy can only be one specified in the referrer_policy_schema.

* **test_expansion_schema**

  The schema used to check if a ```test_expansion``` is valid.
  Each test expansion can only contain fields defined by this schema.

* **subresource_path**

  A 1:1 mapping of a **subresource type** to the URL path of the sub-resource.
  When adding a new sub-resource, a path to an existing file for it also must be specified.


### Test Expansion Patterns

Each field in a test expansion can be in one of the following formats:

* Single match: ```"value"```

* Match any of: ```["value1", "value2", ...]```

* Match all: ```"*"```

#### Example: test expansion in a requirement specification

The following example shows how to restrict the expansion of ```referrer_url``` to  ```origin``` and allow rest of the arrangement to expand (permute) to all possible values. The name field will be the prefix of a generated HTML file name for the test.

```json
    {
      "name": "origin-only",
      "title": "Referrer Policy is set to 'origin-only'",
      "description": "Check that all sub-resources in all cases get only the origin portion of the referrer URL.",
      "specification_url": "https://w3c.github.io/webappsec/specs/referrer-policy/#referrer-policy-state-origin",
      "referrer_policy": "origin",
      "test_expansion": [
        {
          "name": "generic",
          "expansion": "default",
          "source_protocol": "*",
          "target_protocol": "*",
          "delivery_method": "*",
          "redirection": "*",
          "origin": "*",
          "subresource": "*",
          "referrer_url": "origin"
        }
      ]
    }
```

**NOTE:** An expansion is always constructive (inclusive), there isn't a negation operator for explicit exclusion. Be aware that using an empty list ```[]``` matches (expands into) exactly nothing. Tests which are to be excluded should be defined in the ```excluded_tests``` section instead.

A single test expansion pattern, be it a requirement or a suppressed pattern, gets expanded into a list of **selections** as follows:

* Expand each field's pattern (single, any of, or all) to list of allowed values (defined by the ```test_expansion_schema```)

* Permute - Recursively enumerate all **selections** across all fields

Be aware that if there is more than one pattern expanding into a same selection (which also shares the same ```name``` field), the pattern appearing later in the spec JSON will overwrite a previously generated selection. To make sure this is not undetected when generating, set the value of the ```expansion``` field to ```default``` for an expansion appearing earlier and ```override``` for the one appearing later.

A **selection** is a single **test instance** (scenario) with explicit values, for example:

```javascript
var scenario = {
  "referrer_policy": "origin-when-cross-origin",
  "delivery_method": "meta-referrer",
  "redirection": "no-redirect",
  "origin": "cross-origin",
  "source_protocol": "http",
  "target_protocol": "http",
  "subresource": "iframe-tag",
  "subresource_path": "/referrer-policy/generic/subresource/document.py",
  "referrer_url": "origin"
};
```

Essentially, this is what gets generated and defines a single test. The scenario is then evaluated by the ```ReferrerPolicyTestCase``` in JS. For the rest of the arranging part, see the ```./generic/template/``` directory and examine ```./generic/tools/generate.py``` to see how the values for the templates are produced.


Taking the spec JSON, the generator follows this algorithm:

* Expand all ```excluded_tests``` to create a blacklist of selections

* For each specification requirement: Expand the ```test_expansion``` pattern into selections and check each against the blacklist, if not marked as suppresed, generate the test resources for the selection

