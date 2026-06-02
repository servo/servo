Annotation-model: Guidelines for Contributing Tests
===================================================

This document describes the method people should use for authoring tests and
integrating them into the repository.  Anyone is welcome to submit new tests to
this collection.  If you do, please create the tests following the guidelines
below.  Then submit them as a pull request so they can be evaluated

Structure
---------

Tests are organized by major section of the Annotation Model specification.  The
folders associated with these are:

* annotations
* bodiesTargets
* collections
* specificResources
  * selectors
  * states

Within these folders, special files ending with the suffix ".test" provide the source
for the test as a set of declarative assertions about the required shape of the conforming
JSON object.  These files are transformed using a test generation tool into ".html" files
that are then accessed by the Web Platform Test framework.

There are a few other folders that provide supporting materials for the tests:

* common - assertionObjects, conditionObjects, and other supporting materials
* definitions - JSON Schema definitions that can be referenced
* scripts - JavaScript that are included by tests
* tools - supporting scripts and files

NOTE: The files in the definitions folder are expected to be JSON Schema
definitions - basically commonly used concepts that are referenced by other JSON
Schema files in the system.  All of these 'definitions' are preloaded by the
system before any other parts of a test are processed.

Test Cases
----------

Each test is expressed as a simple (or complex) requirement in a test file.
For each section of the document, the requirement is represented as a structure
that describes the nature of the test, and then includes or references minimal
JSON Schema that test the assertions implied by the requirement.

The structure of a test case is defined using a [JSON-LD
Context](JSONtest-v1.jsonld).  That context defines the following terms:

|Keyword        | Values          | Meaning
|---------------|-----------------|---------
|name           | string          | The name of this test for display purposes
|description    | string          | A long self-describing paragraph that explains the purpose of the test and the expected input
|ref            | URI             | An optional reference to the portion of the specification to which the test relates
|testType       | `automated`, `manual`, `ref` | The type of test - this informs [WPT](https://github.com/web-platform-tests/wpt) how the test should be controlled and presented
|skipFailures   | list of strings | An optional list of assertionType values that, if present, should have their test skipped if the result would be "unexpected".  Defaults to the empty list.
|assertions     | list of URI, List @@@ATRISK@@@, or AssertionObject | The ordered collection of tests the input should be run against. See [JSON Schema Usage](#jsonSchema) for the structure of the objects.  URI is relative to the top level folder of the test collection if it has a slash; relative to the current directory if it does not. @@@@ATRISK@@@@ Lists can be nested to define groups of sub-tests.  Assertions / groups can be conditionally skipped.  See [Assertion Lists](#assertionLists) for more details.
|content        | URI or object   | An object containing content to be checked against the referenced assertions, or a URI from which to retrieve that content

Each test case has a suffix of `.test` and a shape like:

<pre>
{
  "@context": "https://www.w3.org/ns/JSONtest-v1.jsonld",
  "name": "Verify annotation conforms to the model",
  "description": "Supply an example annotation that conforms to the basic structure.",
  "ref": "https://www.w3.org/TR/annotation-model/#model",
  "testType": "manual",
  "assertions": [
    "common/has_context.json",
    "common/has_id.json",
    {
      "$schema": "http://json-schema.org/draft-04/schema#",
      "title": "Verify annotation has target",
      "assertionType": "must",
      "expectedResult": "valid",
      "errorMessage": "The object was missing a required 'target' property",
      "type": "object",
      "properties": {
        "target": {
          "anyOf": [
          {
            "type": "string"
          },
          {
            "type": "array",
            "anyOf": [
            {
              "type": "string"
            }
            ]
          }
          ],
            "not": {"type": "object"}
        }
      },
      "required": ["target"]
    }
  ]
}
</pre>

External references are used when the "assertion" is a common one that needs to
be checked on many different test cases (e.g., that there is an @context in the
supplied annotation).

NOTE: The title property of an assertionObject can contain markdown.  This can
help improve readability of the rendered assertions and debugging output.

NOTE: The content property does not yet have a defined use.  One potential use would
be to act as a pointer to a URI that can supply annotations from an implementation.
In that case the URI would take a parameter with the test name as a way of telling
the end point what test is running so it can deliver the right content.

### <a id="assertionLists">Assertion Lists</a> ###

The `assertion` list is an ordered list of assertions that will be evaluated
against the submitted content. The list is *required*, and MUST have at least
one entry. Entries in the list have the following types:

* AssertionObject

An in-line Object as defined in the section [Assertion
Objects](#assertionObjects).
* URI

A relative or absolute URI that references a AssertionObject in a .json file.
If the URI is relative but contains no slashes, then it is considered to be
in the current directory.  If the URI is relative, contains slashes, but
**does not start with a slash** then it is considered relative to the top of
the tree of the current test collection (e.g., `annotation-model`).
* List @@@ATRISK@@@

A nested Assertion List.  While nested Assertion Lists are optional, if one
is present it MUST have at least one entry.  Entries are as in this list.
Assertion Lists can be nested to any depth (but don't do that - it would be
too hard to maintain).


<a id="assertionObjects">Assertion Objects</a>
-----------------

In this collection of tests, Assertion Objects can be contained inline in the
`.test` files or contained in external files with the suffix `.json`.  The
vocabularly and structure is as defined in [JSON Schema
v4](http://json-schema.org/documentation.html) augmented with some additional
properties defined in this section.

In general each JSON Schema definition provided in this test suite should be as
minimal as possible.  This promotes clarity and increases the likelihood that
it is correct.  While it is ---possible--- to create JSON Schema files that
enforce many different requirements on a data model, it increases the
complexity and can also reduce the atomicity of tests / sub-tests (because a
    test ends up testing more than one thing).  Please try to avoid creating
complex JSON Schema.  (A notable exception is the situation where multiple
    properties of a structure are interdependent.)

Tools such as [the JSON Schema Creator](http://jsonschema.net/) may be helpful
in creating schema snippets that can be integrated into JSONtest Assertion
Objects.  Remember that the JSON Schema you create should be as permissive as
possible to address the various shapes that a give property might take (e.g., a
    'foo' might be a string or an object that contains sub-properties that express
    the string, or an array of 1 or more objects with those sub-properties).

In addition to the validation keys defined in JSON Schema v4, Schema files in
this collection are also permitted to use the following keywords:

|Keyword        | Values          | Meaning |
|---------------|-----------------|---------|
|onUnexpectedResult   | `failAndContinue`, `failAndSkip`, `failAndAbort`, `passAndContinue`, `passAndSkip`, `passAndAbort` | Action to take when the result is not as expected. Default is `failAndContinue` |
|assertionType  | `must`, `may`, `should` | Informs the system about the severity of a failure. The default is `must` |
|assertionFile | URI      | An external file that contains an assertion SCHEMA.  When this value is supplied, and local properties will override the ones loaded from the external file.
|errorMessage   | string          | A human readable explanation of what it means if the test fails.  |
|expectedResult | `valid`, `invalid`  | Tells the framework whether validating against this schema is expected to succeed or fail.  The default is `valid` |


### Example Assertion Object ###

<pre>
{
  "$schema": "http://json-schema.org/draft-04/schema#",
    "title": "Verify annotation has @context",
    "type": "object",
    "expectedResult" : "valid",
    "properties": {
      "@context": {
        "anyOf": [
        {
          "type": "string"
        },
        {
          "type": "array",
          "anyOf": [
          {
            "type": "string"
          }
          ]
        }
        ],
          "not": {"type": "object"}
      }
    },
    "required": ["@context"]
}
</pre>

Note that in the case where a feature is *optional* the JSON Schema MUST be
crafted such that if the attribute is permitted to be missing from the content
(so that the result is `true`), but when the attribute is present in the
content it conforms to any requirements.



<a id="conditionObjects">Condition Objects</a>
-----------------

A Condition Object is a sub-class of an Assertion Object.  It allows the
specification of the evaluation strategy for the assertions referenced by the
object.  An object is a Condition Object IFF it has a `assertions` property. In
this case, there MUST NOT be an `assertionFile` property.


|Keyword        | Values          | Meaning |
|---------------|-----------------|---------|
|compareWith    | `and`, `or` | How should the result of any referenced assertions be compared.  Defaults to `and`.  Note that this implies there is also an assertions property with a nested list of assertions to compare. |
|assertions     | a list of assertions as in a Test Case above. This is required if there is a compareWith property |


An example of a test that would pass if there were an `@context` OR there were an `@id`:

<pre>
{
  "@context": "https://www.w3.org/ns/JSONtest-v1.jsonld",
    "name": "A test that has an 'or' clause",
    "description": "A complex test that uses or-ing among a list of assertions",
    "ref": "https://www.w3.org/TR/annotation-model/#model",
    "testType": "manual",
    "assertions": [
    { "$schema": "http://json-schema.org/draft-04/schema#",
      "title": "must have context or id",
      "description": "A more complex example that allows one of many options to pass",
      "assertions": [
      { "title": "Condition Object",
        "description": "A pseudo-test that will get a result from the aggregate of its children",
        "assertionType": "must",
        "expectedResult": "valid",
        "errorMessage": "Error: None of the various options were present",
        "compareWith": "or",
        "assertions": [
          "common/has_context.json",
        "common/has_id.json"
        ]
      }
      ]
    }
    ]
}
</pre>


Command Line Tools
------------------

### Building the Test Files ###

The actual .html test case files are generated using the script
tools/make_tests.py.  This script will search the directory heriarchy looking for
files ending on `.test` and creating `.html` files from them using the template in
the tools folder.  If you want to regenerate the examples too, supply the
`--examples` option to the script.

Note that when submitting tests to the repository, the `.html` versions must be
included.

### Testing the Tests ###

### Driving Tests with Input Files ###

Complex Examples
----------------

This section is a collection of more complex examples to illustrate the
expressive power of the [Assertion List](#assertionLists) structure.  These can
be used as templates for creating actual `.test` files.

### Including and Overriding an Assertion ###

Assertions can be contained in external `.json` files.  It is possible for an
object in an Assertion List to include the external file and override one or
more of its properties:

<pre>
{
  "@context": "https://www.w3.org/ns/JSONtest-v1.jsonld",
    "name": "Permit no target property",
    "description": "Ensure there is no 'target' property when there is a 'randomName' property in the Annotation",
    "assertions": [
    {
      "$schema": "http://json-schema.org/draft-04/schema#",
      "title": "Verify annotation has randomName",
      "type": "object",
      "properties": {
        "randomName": {
          "type": "string"
        }
      },
      "required": ["randomName"]
    },
    { "assertionFile" : "common/target.json",
      "title" : "Require target to be missing",
      "expectedResult" : "invalid",
      "errorMessage" : "The target MUST not be present when 'randomName' is also present",
    }
    ]
}
</pre>

### Nested Assertion Collections with Skip ###

Assertion Lists can be nested within Assertion Lists.  This feature, combined
with the `onUnexpectedResult` property, makes it possible to skip a collection
of tests when an assertion in the list is not satisfied:

<pre>
{
  "@context": "https://www.w3.org/ns/JSONtest-v1.jsonld",
    "name": "If there is no 'target' property, skip some tests",
    "description": "When 'target' is not present, other properties related to 'target' are not required",
    "assertions": [
      "common/context.json",
    [
    { "assertionFile" : "common/target.json",
      "errorMessage" : "Target was not present so skip the rest of this section",
      "onUnexpectedResult" : "failAndSkip"
    },
    "sometest.json",
    "sometest2.json",
    "sometest3.json"
    ]
    ]
} ;
</pre>

### Assertion that finds a specific @context Value ###

Sometimes you want a property to be flexible, but to have one and only one of a
specific value.  This is especially true with, for example, @context in JSON-LD.
One way you might do this is:

<pre>
{
  "$schema": "http://json-schema.org/draft-04/schema#",
    "title": "Verify a specific @context",
    "type": "object",
    "expectedResult" : "valid",
    "properties": {
      "@context": {
        "anyOf": [
        {
          "type": "string"
            "enum": [ "http://www.w3.org/ns/anno.jsonld" ]
        },
        {
          "type": "array",
          "minitems": "1",
          "uniqueItems": true,
          "additionalItems": true,
          "items" : [
          { "type": "string",
            "enum": [ "http://www.w3.org/ns/anno.jsonld" ]
          }
          ]
        }
        ],
          "not": {"type": "object"}
      }
    },
    "required": ["@context"]
}

</pre>
