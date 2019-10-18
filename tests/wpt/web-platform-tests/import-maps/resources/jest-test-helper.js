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

// Sort keys and then stringify for comparison.
function stringifyImportMap(importMap) {
  function getKeys(m) {
    if (typeof m !== 'object')
      return [];

    let keys = [];
    for (const key in m) {
      keys.push(key);
      keys = keys.concat(getKeys(m[key]));
    }
    return keys;
  }
  return JSON.stringify(importMap, getKeys(importMap).sort());
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
          stringifyImportMap(actualParsedImportMap),
          stringifyImportMap(expected)
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
  // We can't test data: base URLs because <base> rejects data: URLs.
  if (new URL(mapBaseURL).protocol === 'data:') {
    throw Error('test helper does not support data: base URLs');
  }

  const iframe = document.createElement('iframe');
  document.body.appendChild(iframe);
  iframe.contentDocument.write(`
    <base href="${mapBaseURL}">
    <script>
    var scriptError;
    var windowError;
    function onScriptError(event) {
      scriptError = event.error;
    }
    function onWindowError(event) {
      windowError = event.error;
      return false;
    }
    window.addEventListener('error', onWindowError);
    </sc` + `ript>
    <script type="importmap" onerror="onScriptError(event)">
    ${mapString}
    </sc` + `ript>
  `);
  iframe.contentDocument.close();

  // Rethrow window's error event.
  if (iframe.contentWindow.windowError) {
    throw iframe.contentWindow.windowError;
  }

  return iframe;
}

// URL resolution is tested using Chromium's `internals`.
// TODO(hiroshige): Remove the Chromium-specific dependency.
function resolve(specifier, map, baseURL) {
  return internals.resolveModuleSpecifier(specifier,
                                          baseURL,
                                          map.contentDocument);
}
