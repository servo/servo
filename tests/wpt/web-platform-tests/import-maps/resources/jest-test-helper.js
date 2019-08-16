// Hacky glue code to run Jest-based tests as WPT tests.
// TODO(https://github.com/WICG/import-maps/issues/170): Consider better ways
// to write and run tests.

setup({allow_uncaught_exception : true});

const exports = {};

function require(name) {
  return Object.assign({
    'URL': URL,
    'parseFromString': parseFromString,
    'resolve': resolve,
    'BUILT_IN_MODULE_SCHEME': 'std'
  }, exports);
}

function expect(v) {
  return {
    toMatchURL: expected => assert_equals(v, expected),
    toThrow: expected => {
      if (expected.test && expected.test('not yet implemented')) {
        // We override /not yet implemented/ expectation.
        assert_throws(TypeError(), v);
      } else {
        assert_throws(expected(), v);
      }
    },
    toEqual: expected => {
      if (v.localName === 'iframe') {
        // `v` is the result of parseFromString(), and thus toEqual() is
        // expected to compare parsed import maps.
        // We sort keys when stringifying for normalization.
        const actualParsedImportMap = JSON.parse(
            internals.getParsedImportMap(v.contentDocument));
        assert_equals(
          JSON.stringify(actualParsedImportMap,
                         Object.keys(actualParsedImportMap).sort()),
          JSON.stringify(expected.imports,
                         Object.keys(expected.imports).sort())
        );
      } else {
        assert_object_equals(v, expected);
      }
    }
  };
}

expect.toMatchURL = expected => expected;

const test_harness_test = test;
test = it;

let current_message = '';
function describe(message, f) {
  const old = current_message;
  if (current_message !== '') {
    current_message += ' / ';
  }
  current_message += message;
  f();
  current_message = old;
}
function it(message, f) {
  const old = current_message;
  if (current_message !== '') {
    current_message += ' / ';
  }
  current_message += message;
  test_harness_test(t => t.step_func(f)(), current_message);
  current_message = old;
}

// Creates a new Document (via <iframe>) and add an inline import map.
// Currently document.write() is used to make everything synchronous, which
// is just needed for running the existing Jest-based tests easily.
function parseFromString(mapString, mapBaseURL) {
  const iframe = document.createElement('iframe');
  document.body.appendChild(iframe);
  iframe.contentDocument.write(`
    <base href="${mapBaseURL}">
    <script>
    let isError = false;
    function onError() {
      isError = true;
    }
    </sc` + `ript>
    <script type="importmap" onerror="onError()">
    ${mapString}
    </sc` + `ript>
  `);
  iframe.contentDocument.close();
  return iframe;
}

// URL resolution is tested using Chromium's `internals`.
// TODO(hiroshige): Remove the Chromium-specific dependency.
function resolve(specifier, map, baseURL) {
  return internals.resolveModuleSpecifier(specifier,
                                          baseURL,
                                          map.contentDocument);
}
