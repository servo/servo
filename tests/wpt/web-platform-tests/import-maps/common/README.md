# Import maps test JSON format

In this directory, test inputs and expectations are expressed as JSON files.
This is in order to share the same JSON files between WPT tests and Jest-based
tests for the reference JavaScript implementation at [WICG repository](https://github.com/WICG/import-maps/tree/master/reference-implementation).

## Basics

A **test object** describes a set of parameters (import maps and base URLs) and test expectations.
Test expectations can be:

- Expected resulting URLs for specifiers (resolution tests), or
- Expected parsed import maps (parsing tests).

Each JSON file under [resources/](resources/) directory consists of a test object.
A minimum test object would be:

```json
{
  "name": "Main test name",
  "importMapBaseURL": "https://example.com/import-map-base-url/index.html",
  "importMap": {
    "imports": {
      "a": "/mapped-a.mjs"
    }
  },
  "baseURL": "https://example.com/base-url/app.mjs",
  "expectedResults": {
    "a": "https://example.com/mapped-a.mjs",
    "b": null
  }
}
```

Required fields:

- `name`: Test name.
    - In WPT tests, this is used for the test name of `promise_test()` together with specifier to be resolved, like `"Main test name: a"`.
- `importMap` (object or string): the import map to be attached.
- `importMapBaseURL` (string): the base URL used for [parsing the import map](https://wicg.github.io/import-maps/#parse-an-import-map-string).
- (resolution tests only) `expectedResults` (object; string to (string or null)): resolution test cases.
    - The keys are specifiers to be resolved.
    - The values are expected resolved URLs. If `null`, resolution should fail.
- (resolution tests only) `baseURL` (string): the base URL used in [resolving a specifier](https://wicg.github.io/import-maps/#resolve-a-module-specifier) for each specifiers.
- (parsing tests only) `expectedParsedImportMap` (object): the expected parsed import map for parsing test cases.

Optional fields:

- `link` and `details` can be used for e.g. linking to specs or adding more detailed descriptions.
    - Currently they are simply ignored by the WPT test helper.

## Nesting and inheritance

We can organize tests by nesting test objects.
A test object can contain child test objects (*subtests*) using `tests` field.
The Keys of the `tests` value are the names of subtests, and values are test objects.

For example:

```json
{
  "name": "Main test name",
  "importMapBaseURL": "https://example.com/import-map-base-url/index.html",
  "importMap": {
    "imports": {
      "a": "/mapped-a.mjs"
    }
  },
  "tests": {
    "Subtest1": {
      "baseURL": "https://example.com/base-url1/app.mjs",
      "expectedResults": { "a": "https://example.com/mapped-a.mjs" }
    },
    "Subtest2": {
      "baseURL": "https://example.com/base-url2/app.mjs",
      "expectedResults": { "b": null }
    }
  }
}
```

The top-level test object contains two sub test objects, named as `Subtest1` and `Subtest2`, respectively.

Child test objects inherit fields from their parent test object.
In the example above, the child test objects specifies `baseURL` fields, while they inherits other fields (e.g. `importMapBaseURL`) from the top-level test object.
