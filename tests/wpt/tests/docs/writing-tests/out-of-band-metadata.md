# Out-of-band Metadata

WPT uses two kinds of [YAML](https://yaml.org/)-formatted text files to declare
nonessential information about tests: `META.yml` and `WEB_FEATURES.yml`.

## `META.yml`

Files with this name may appear in any directory of the web-platform-tests.
They may define any of the following properties:

- `spec` - a link to the specification covered by the tests in the directory
- `suggested_reviewers` - a list of GitHub account username belonging to
  people who are notified when pull requests modify files in the directory

## `WEB_FEATURES.yml`

Files with this name may appear in any directory that includes tests. They
store a mapping between the tests in the local directory and [the
web-features](https://github.com/web-platform-dx/web-features) which those
tests validate.

They define one property, `features`, which is a list of rules that relate one
web-feature to one or more tests in that directory. Each mapping rule includes
two properties: `name` (whose value is the string identifier of a
web-feature[^1]) and `files` (whose value is either the string value `**` or a
list of file pattern strings).

<details>
  <summary>Formal [CDDL](https://datatracker.ietf.org/doc/html/rfc8610) schema definition</summary>

```
MappingRules = {
  features: [*MappingRule],
}

MappingRule = {
  name: text,
  files: [*FilePattern],
}

FilePattern = text .regexp "!?[A-Za-z0-9_*.-]+"
```

</details>

These files are used to generate a single JSON-formatted manifest file which
relates web-feature IDs to literal test file names (rather than lists of
patterns). Such a manifest is automatically generated for every commit made on
WPT's main development branch and included in [a corresponding
release](https://github.com/web-platform-tests/wpt/releases) under the name
`WEB_FEATURES_MANIFEST` (available in a number of encodings).

### File patterns

If the `files` property takes the string value `**`, this signifies that all
tests in the current directory and all subdirectories (if present) belong to
the corresponding web-feature.

If the `files` property is a list of string values, the strings are interpreted
as file patterns. Each contributes to the definition of a set of files which
should be associated with the corresponding web-feature. While these "patterns"
may be literal file names, they also support the following operators which
alter their meaning:

- An asterisk appearing anywhere in the string (e.g. `foo-*.js`) is a
  placeholder for zero or more other characters. Patterns using a star can
  therefore describe multiple files.
- A leading exclamation point (e.g. `!foobar.html`) means, "*exclude* any file
  which matches the pattern." This operator is intended to refine rules which
  also include patterns with the asterisk operator.

If the `*` pattern appears as a list item, it will match all tests in the
current directory; it will not match any subdirectories. There is no mechanism
for matching specific subdirectories (only for matching *all* subdirectories
via `**`). To define mappings for files in a given subdirectory, write mapping
rules in a `WEB_FEATURES.yml` file within that subdirectory.

The elements of the `files` list are applied from top to bottom to produce the
set of files which belongs to a given web-feature. (The behavior of this list
of file patterns is similar to how [the Git version control system interprets
`.gitignore` files](https://git-scm.com/docs/gitignore).)

### Caveat: Pattern Matching

Pattern matching allows classifiers to be resilient to expected changes in
directory contents. This often occurs when file names have predictable file
names according to:

- their intent (e.g. `shape-function-valid.html` and
  `shape-function-invalid.html`)
- their membership in a sequence (e.g. `float-023.xht`, `float-024.xht`, etc.)
- [their tentative
  status](https://web-platform-tests.org/writing-tests/file-names.html) (e.g.
  `offset_and_page_after_dispatch.tentative.html`)

While the asterisk operator can improve concision and robustness in such cases,
it also makes it easy to write classifiers that will include unrelated new
tests. If there is a reasonable chance that future contributions may
unintentionally match a given pattern, then a more restrictive pattern (or a
list of literal file names) is likely preferable.

For instance, the pattern `b*` might match a desired set of tests today, but it
is susceptible to matching unrelated new tests added later. If a more
restrictive pattern like `border-*` will suffice, it is generally preferable as
a safer alternative.

### Caveat: Cross-cutting tests

It is common for tests in web-platform-tests to validate the behavior of more
than one web-feature. Rather than including such tests in multiple
classifications, it is generally preferable to avoid classifying them at all.
(It may be possible to refactor tests like this into multiple files that each
focus on a single web-feature, but that work is not considered a high
priority.)

### Example 1: Mapping an entire directory to a single web-feature

For example, if the directory named `fetch/` contained only tests for [the
`fetch`
web-feature](https://web-platform-dx.github.io/web-features-explorer/features/fetch/),
then that directory might include a `WEB_FEATURES.yml` file whose content
appears as follows:

```yaml
features:
- name: fetch
  files: "**"
```

### Example 2: Mapping tests within a directory to many web-features

Given a directory with the following entries:

- `crashtests/`
- `resources/`
- `META.yml`
- `WEB_FEATURES.yml`
- `aspect-ratio1.html`
- `aspect-ratio2.html`
- `ar1.html`
- `ar1-ref.html`
- `ar2.html`
- `ar2-ref.html`
- `box-sizing-1.html`
- `box-sizing-2.html`
- `box-sizing-3.js`
- `z-index.html`

The contents of the file named `WEB_FEATURES.yml` might appear as follows:

```yaml
features:
- name: aspect-ratio
  files:
  - "*"              # This line includes all test files in the directory
  - "!box-sizing-*"  # This line excludes all test files whose name begins with "box-sizing"
  - "!z-index.html"  # This line excludes the test file named "z-index.html"
- name: box-sizing
  files:
  - box-sizing-*  # This line includes all test files whose name begins with "box-sizing"
- name: z-index
  files:
  - z-index.html  # This line includes the test file named "z-index.html"
```

[^1]: The web-feature identifier is distinct from the web-feature name.
      Although the property is "name", its value should actually be the
      identifier.
