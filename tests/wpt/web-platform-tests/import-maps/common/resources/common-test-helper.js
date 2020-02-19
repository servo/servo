setup({allow_uncaught_exception : true});

// Creates a new Document (via <iframe>) and add an inline import map.
function parse(importMap, importMapBaseURL) {
  return new Promise(resolve => {
    const importMapString = JSON.stringify(importMap);
    const iframe = document.createElement('iframe');

    window.addEventListener('message', event => {
        // Parsing result is saved here and checked later, rather than
        // rejecting the promise on errors.
        iframe.parseImportMapResult = event.data.type;
        resolve(iframe);
      },
      {once: true});

    const testHTML = `
      <script>
      // Handle errors around fetching, parsing and registering import maps.
      let registrationResult;
      const onScriptError = event => {
        registrationResult = {type: 'FetchError', error: event.error};
        return false;
      };
      const windowErrorHandler = event => {
        registrationResult = {type: 'ParseError', error: event.error};
        return false;
      };
      window.addEventListener('error', windowErrorHandler);
      window.addEventListener('load', event => {
        if (!registrationResult) {
          registrationResult = {type: 'Success'};
        }
        window.removeEventListener('error', windowErrorHandler);
        parent.postMessage(registrationResult, '*');
      });

      // Handle specifier resolution requests from the parent frame.
      window.addEventListener('message', event => {
        try {
          if (event.data.action === 'resolve') {
            // URL resolution is tested using Chromium's internals.
            // TODO(hiroshige): Remove the Chromium-specific dependency.
            const result = internals.resolveModuleSpecifier(
                event.data.specifier,
                event.data.baseURL,
                document);
            parent.postMessage({type: 'ResolutionSuccess', result: result}, '*');
         } else if (event.data.action === 'getParsedImportMap') {
           parent.postMessage({
               type: 'GetParsedImportMapSuccess',
               result: internals.getParsedImportMap(document)}, '*');
         } else {
           parent.postMessage({
               type: 'Failure',
               result: "Invalid Action: " + event.data.action}, '*');
         }
        } catch (e) {
          // We post error names instead of error objects themselves and
          // re-create error objects later, to avoid issues around serializing
          // error objects which is a quite new feature.
          parent.postMessage({type: 'Failure', result: e.name}, '*');
        }
      });
      </script>
      <script type="importmap" onerror="onScriptError(event)">
      ${importMapString}
      </script>
    `;

    if (new URL(importMapBaseURL).protocol === 'data:') {
      iframe.src = 'data:text/html;base64,' + btoa(testHTML);
    } else {
      iframe.srcdoc = `<base href="${importMapBaseURL}">` + testHTML;
    }

    document.body.appendChild(iframe);

  });
}

// Returns a promise that is resolved with the resulting URL.
function resolve(specifier, parsedImportMap, baseURL) {
  return new Promise((resolve, reject) => {
    window.addEventListener('message', event => {
        if (event.data.type === 'ResolutionSuccess') {
          resolve(event.data.result);
        } else if (event.data.type === 'Failure') {
          if (event.data.result === 'TypeError') {
            reject(new TypeError());
          } else {
            reject(new Error(event.data.result));
          }
        } else {
          assert_unreached('Invalid message: ' + event.data.type);
        }
      },
      {once: true});

    parsedImportMap.contentWindow.postMessage(
        {action: "resolve", specifier: specifier, baseURL: baseURL}, '*');
  });
}

// Returns a promise that is resolved with a serialized string of
// a parsed import map JSON object.
function getParsedImportMap(parsedImportMap) {
  return new Promise((resolve, reject) => {
    window.addEventListener('message', event => {
        resolve(event.data.result);
      },
      {once: true});

    parsedImportMap.contentWindow.postMessage(
        {action: "getParsedImportMap"}, '*');
  });
}

function assert_no_extra_properties(object, expectedProperties, description) {
  for (const actualProperty in object) {
    assert_true(expectedProperties.indexOf(actualProperty) !== -1,
        description + ": unexpected property " + actualProperty);
  }
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

async function runTests(j) {
  const tests = j.tests;
  delete j.tests;

  if (j.hasOwnProperty('importMap')) {
    assert_own_property(j, 'importMap');
    assert_own_property(j, 'importMapBaseURL');
    j.parsedImportMap = await parse(j.importMap, j.importMapBaseURL);
    delete j.importMap;
    delete j.importMapBaseURL;
  }

  assert_no_extra_properties(
      j,
      ['expectedResults', 'expectedParsedImportMap',
      'baseURL', 'name', 'parsedImportMap',
      'importMap', 'importMapBaseURL',
      'link', 'details'],
      j.name);

  if (tests) {
    // Nested node.
    for (const testName in tests) {
      let fullTestName = testName;
      if (j.name) {
        fullTestName = j.name + ': ' + testName;
      }
      tests[testName].name = fullTestName;
      const k = Object.assign(Object.assign({}, j), tests[testName]);
      await runTests(k);
    }
  } else {
    // Leaf node.
    for (const key of ['parsedImportMap', 'name']) {
      assert_own_property(j, key, j.name);
    }
    assert_true(j.hasOwnProperty('expectedResults') ||
                j.hasOwnProperty('expectedParsedImportMap'),
                'expectedResults or expectedParsedImportMap should exist');

    // Resolution tests.
    if (j.hasOwnProperty('expectedResults')) {
      assert_own_property(j, 'baseURL');
      assert_equals(
          j.parsedImportMap.parseImportMapResult,
          "Success",
          "Import map registration should be successful for resolution tests");
      for (const specifier in j.expectedResults) {
        const expected = j.expectedResults[specifier];
        promise_test(async t => {
            if (expected === null) {
              return promise_rejects_js(t, TypeError,
                  resolve(specifier, j.parsedImportMap, j.baseURL));
            } else {
              // Should be resolved to `expected`.
              const actual = await resolve(
                  specifier, j.parsedImportMap, j.baseURL);
              assert_equals(actual, expected);
            }
          },
          j.name + ': ' + specifier);
      }
    }

    // Parsing tests.
    if (j.hasOwnProperty('expectedParsedImportMap')) {
      promise_test(async t => {
        if (j.expectedParsedImportMap === null) {
          assert_equals(j.parsedImportMap.parseImportMapResult, "ParseError");
        } else {
          const actualParsedImportMap =
              await getParsedImportMap(j.parsedImportMap);
          assert_equals(stringifyImportMap(JSON.parse(actualParsedImportMap)),
                        stringifyImportMap(j.expectedParsedImportMap));
        }
      }, j.name);
    }

  }
}

export async function runTestsFromJSON(jsonURL) {
  const response = await fetch(jsonURL);
  const json = await response.json();
  await runTests(json);
}
