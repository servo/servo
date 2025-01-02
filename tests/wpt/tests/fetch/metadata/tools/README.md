# Fetch Metadata test generation framework

This directory defines a command-line tool for procedurally generating WPT
tests.

## Motivation

Many features of the web platform involve the browser making one or more HTTP
requests to remote servers. Only some aspects of these requests are specified
within the standard that defines the relevant feature. Other aspects are
specified by external standards which span the entire platform (e.g. [Fetch
Metadata Request Headers](https://w3c.github.io/webappsec-fetch-metadata/)).

This state of affairs makes it difficult to maintain test coverage for two
reasons:

- When a new feature introduces a new kind of web request, it must be verified
  to integrate with every cross-cutting standard.
- When a new cross-cutting standard is introduced, it must be verified to
  integrate with every kind of web request.

The tool in this directory attempts to reduce this tension. It allows
maintainers to express instructions for making web requests in an abstract
sense. These generic instructions can be reused by to produce a different suite
of tests for each cross-cutting feature.

When a new kind of request is proposed, a single generic template can be
defined here. This will provide the maintainers of all cross-cutting features
with clear instruction on how to extend their test suite with the new feature.

Similarly, when a new cross-cutting feature is proposed, the authors can use
this tool to build a test suite which spans the entire platform.

## Build script

To generate the Fetch Metadata tests, run `./wpt update-built --include fetch`
in the root of the repository.

## Configuration

The test generation tool requires a YAML-formatted configuration file as its
input. The file should define a dictionary with the following keys:

- `templates` - a string describing the filesystem path from which template
  files should be loaded
- `output_directory` - a string describing the filesystem path where the
  generated test files should be written
- `cases` - a list of dictionaries describing how the test templates should be
  expanded with individual subtests; each dictionary should have the following
  keys:
  - `all_subtests` - properties which should be defined for every expansion
  - `common_axis` - a list of dictionaries
  - `template_axes` - a dictionary relating template names to properties that
    should be used when expanding that particular template

Internally, the tool creates a set of "subtests" for each template. This set is
the Cartesian product of the `common_axis` and the given template's entry in
the `template_axes` dictionary. It uses this set of subtests to expand the
template, creating an output file. Refer to the next section for a concrete
example of how the expansion is performed.

In general, the tool will output a single file for each template. However, the
`filename_flags` attribute has special semantics. It is used to separate
subtests for the same template file. This is intended to accommodate [the
web-platform-test's filename-based
conventions](https://web-platform-tests.org/writing-tests/file-names.html).

For instance, when `.https` is present in a test file's name, the WPT test
harness will load that test using the HTTPS protocol. Subtests which include
the value `https` in the `filename_flags` property will be expanded using the
appropriate template but written to a distinct file whose name includes
`.https`.

The generation tool requires that the configuration file references every
template in the `templates` directory. Because templates and configuration
files may be contributed by different people, this requirement ensures that
configuration authors are aware of all available templates. Some templates may
not be relevant for some features; in those cases, the configuration file can
include an empty array for the template's entry in the `template_axes`
dictionary (as in `template3.html` in the example which follows).

## Expansion example

In the following example configuration file, `a`, `b`, `s`, `w`, `x`, `y`, and
`z` all represent associative arrays.

```yaml
templates: path/to/templates
output_directory: path/to/output
cases:
  - every_subtest: s
    common_axis: [a, b]
    template_axes:
      template1.html: [w]
      template2.html: [x, y, z]
      template3.html: []
```

When run with such a configuration file, the tool would generate two files,
expanded with data as described below (where `(a, b)` represents the union of
`a` and `b`):

    template1.html: [(a, w), (b, w)]
    template2.html: [(a, x), (b, x), (a, y), (b, y), (a, z), (b, z)]
    template3.html: (zero tests; not expanded)

## Design Considerations

**Efficiency of generated output** The tool is capable of generating a large
number of tests given a small amount of input. Naively structured, this could
result in test suites which take large amount of time and computational
resources to complete. The tool has been designed to help authors structure the
generated output to reduce these resource requirements.

**Literalness of generated output** Because the generated output is how most
people will interact with the tests, it is important that it be approachable.
This tool avoids outputting abstractions which would frustrate attempts to read
the source code or step through its execution environment.

**Simplicity** The test generation logic itself was written to be approachable.
This makes it easier to anticipate how the tool will behave with new input, and
it lowers the bar for others to contribute improvements.

Non-goals include conciseness of template files (verbosity makes the potential
expansions more predictable) and conciseness of generated output (verbosity
aids in the interpretation of results).
