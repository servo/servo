# WMAS2017 ECMA Integration

## Generating Tests

Clone the ECMAScript 5 tests into the working directory

```
$ git clone git@github.com:tc39/test262.git -b es5-tests
```

Working directory should look like this

```
generate-tests.js
test262
test-template.html
webplatform-adapter.js
```

Generate the tests by running

```
$ node generate-tests.js
```

Generated tests are placed in `ecmascript` directory. Copy this
directory into the top level directory of the Web Platform Tests
hierarchy in order for the Web Platform Test Runner to run them.

## Test generation parameters

```
$ node generate-tests.js < test262-repo-dir > < output-dir >
```

You can specify where the test262 repository is located and where the
generated tests should be put in by passing the paths to the
generator script as shown above.

## Excluded tests

The following tests are automatically excluded, because they are
causing the browser to freeze.

```
ch15/15.4/15.4.4/15.4.4.15/15.4.4.15-3-14.js
ch15/15.4/15.4.4/15.4.4.18/15.4.4.18-3-14.js
ch15/15.4/15.4.4/15.4.4.20/15.4.4.20-3-14.js
ch15/15.4/15.4.4/15.4.4.21/15.4.4.21-3-14.js
ch15/15.4/15.4.4/15.4.4.22/15.4.4.22-3-14.js
```
