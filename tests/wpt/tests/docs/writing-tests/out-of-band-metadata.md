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

They define one property, `rules`, which is a list of rules that relate one or
more tests in that directory to one or more web-feature. Each mapping rule
includes one property. That property's name is a file-pattern string (matching
one or more tests), and that property's value is a list of web-feature IDs.

<details>
  <summary>Formal [CDDL](https://datatracker.ietf.org/doc/html/rfc8610) schema definition</summary>

```
MappingRules = {
  rules: [*MappingRule],
}

MappingRule = (
  ConciseMappingRule //
  ExtendedMappingRule
)

ConciseMappingRule = {
  *FilePattern => [*text]
}

ExtendedMappingRule = {
  *FilePattern => {
    ids: [*text]
  }
}

FilePattern = text .regexp "[A-Za-z0-9_*.-]+"
```

</details>

These files are used to generate a single JSON-formatted manifest file which
relates web-feature IDs to literal test file names (rather than lists of
patterns). Such a manifest is automatically generated for every commit made on
WPT's main development branch and included in [a corresponding
release](https://github.com/web-platform-tests/wpt/releases) under the name
`WEB_FEATURES_MANIFEST` (available in a number of encodings).

### File patterns

If the property name takes the string value `**`, this signifies that all tests
in the current directory and all subdirectories (if present) belong to the
corresponding web-features.

Any other value of the property name is interpreted as a file pattern. The
matching set of files will be associated with the corresponding web-feature.
While these "patterns" may be literal file names, they also support the "star"
(or "glob") operators. An asterisk appearing anywhere in the string (e.g.
`foo-*.js`) is a placeholder for zero or more other characters. Patterns using
a star can therefore describe multiple files in the same directory as the
`WEB_FEATURES.yml` file.

There is no mechanism for matching specific subdirectories (only for matching
*all* subdirectories via `**`). To define mappings for files in a given
subdirectory, write mapping rules in a `WEB_FEATURES.yml` file within that
subdirectory.

These rules are interpreted from top to bottom. When a test file matches a
given rule, it is no longer considered when interpreting subsequent rules. For
this reason, if an author intends for a test file (or set of test files) to be
associated with multiple web-features, the author should write a single rule
that associates that file (or set of files) with all the desired web-features.

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
rules:
- "**": [fetch]
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
rules:
# The following rule includes all test files whose name begins with "box-sizing":
- box-sizing-*: [box-sizing]

# The following rule includes the test file named "z-index.html"
- z-index.html: [z-index]

# The following rule matches all test files which have not been matched above:
- "*": [aspect-ratio]
```

### Example 3: Mapping the same test file(s) to more than one web-feature

The following `WEB_FEATURES.yml` file will associate the test files bearing the
`foo-` prefix with the web-feature `grape`:

```yaml
rules:
- foo-*: [grape]
```

The following `WEB_FEATURES.yml` file is equivalent because the test files
prefixed with `foo-` are matched by the first rule and ignored by the
subsequent rule-processing logic:

```yaml
rules:
# the following rule matches all test files:
- foo-*: [grape]

# there are no unmatched files when this rule is encountered, so the following
# rule matches zero test files:
- foo-*: [orange]
```

To associate the `foo-`-prefixed test files with the web-features named `grape`
*and* the `web-feature` named `orange`, one must declare both web-feature IDs
in the same rule, as in the following `WEB_FEATURES.yml` file:

```yaml
rules:
- foo-*: [grape, orange]
```

### Example 4: Excluding test files from all web-features

If the list value reserved for web-feature IDs is an empty list, then the test
files matched by the rule will not be associated with any web-feature. This is
helpful when the majority of test files within a directory should be classified
but a small number should not.

For example, consider a directory where the test-file named `c.html` should not
be associated with any web-feature, but all other web-features should be
associated with the `background` web-feature. Authors may write rules "around"
the excluded test file:

```yaml
rules:
- a.html: [background]
- b.html: [background]
- d.html: [background]
- e.html: [background]
```

...but the above file does not clearly reflect their intent (that is, "all
files except `c.html`). Instead, authors may write explicit rules to exclude
some test files and follow that with a smaller set of more expansive rules:

```yaml
rules:
- c.html: []
- "*": [background]
```

The `*` pattern will not match `c.html` in this context because that test file
has already been matched by a preceding rule.
