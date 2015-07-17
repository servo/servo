CSS tests require a specific naming convention. This is also a good,
but not mandatory, style to use for other tests.

## File Name

The file name format is ```test-topic-###.ext``` where `test-topic`
somewhat describes the test, `###` is a zero-filled number used to
keep the file names unique, and `ext` is typically either
`html` or `xht`.

### test-topic

`test-topic` is a short identifier that describes the test. The
`test-topic` should avoid conjunctions, articles, and prepositions.
It is a file name, not an English phrase: it should be as concise
as possible.

Examples:
```
    margin-collapsing-###.ext
    border-solid-###.ext
    float-clear-###.ext
```

### `###`

`###` is a zero-filled number used to keep the file names unique when
files have the same test-topic name.

Note: The number format is limited to 999 cases. If you go over this
number it is recommended that you reevaluate your test-topic name.

For example, in the case of margin-collapsing there are multiple
cases so each case could have the same test-topic but different
numbers:

```
    margin-collapsing-001.xht
    margin-collapsing-002.xht
    margin-collapsing-003.xht
```

There may also be a letter affixed after the number, which can be
used to indicate variants of a test.

For example, ```float-wrap-001l.xht``` and ```float-wrap-001r.xht```
might be left and right variants of a float test.

If tests using both the unsuffixed number and the suffixed number
exist, the suffixed tests must be subsets of the unsuffixed test.

For example, if ```bidi-004``` and ```bidi-004a``` both exist,
```bidi-004a``` must be a subset of ```bidi-004```.

If the unsuffixed test is strictly the union of the suffixed tests,
i.e. covers all aspects of the suffixed tests (such that a user agent
passing the unsuffixed test will, by design, pass all the suffixed
tests), then the unsuffixed test should be marked with the combo flag.

If ```bidi-004a``` and ```bidi-004b``` cover all aspects of ```bidi-
004``` (except their interaction), then bidi-004 should be given the
combo flag.

### ext

`ext` is the file extension or format of the file.
For XHTML test files, it should be `xht`.
For HTML (non-XML) test files, it should be `html`.
