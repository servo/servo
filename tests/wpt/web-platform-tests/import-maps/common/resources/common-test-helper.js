setup({allow_uncaught_exception : true});

// Creates a new Document (via <iframe>) and add an inline import map.
function parse(importMap, importMapBaseURL) {
  return new Promise((resolve, reject) => {
    const importMapString = JSON.stringify(importMap);
    const iframe = document.createElement('iframe');

    window.addEventListener('message', event => {
        if (event.data.type === 'Success') {
          resolve(iframe);
        } else {
          // Currently we don't distinguish fetch errors and parse errors.
          reject(event.data.error);
        }
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
          // URL resolution is tested using Chromium's internals.
          // TODO(hiroshige): Remove the Chromium-specific dependency.
          const result = internals.resolveModuleSpecifier(
              event.data.specifier,
              event.data.baseURL,
              document);
          parent.postMessage({type: 'ResolutionSuccess', result: result}, '*');
        } catch (e) {
          // We post error names instead of error objects themselves and
          // re-create error objects later, to avoid issues around serializing
          // error objects which is a quite new feature.
          parent.postMessage({type: 'ResolutionFailure', result: e.name}, '*');
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
        } else if (event.data.type === 'ResolutionFailure') {
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
        {specifier: specifier, baseURL: baseURL}, '*');
  });
}

function assert_no_extra_properties(object, expectedProperties, description) {
  for (const actualProperty in object) {
    assert_true(expectedProperties.indexOf(actualProperty) !== -1,
        description + ": unexpected property " + actualProperty);
  }
}

async function runTests(j) {
  const tests = j.tests;
  delete j.tests;

  if (j.importMap) {
    assert_own_property(j, 'importMap');
    assert_own_property(j, 'importMapBaseURL');
    j.parsedImportMap = await parse(j.importMap, j.importMapBaseURL);
    delete j.importMap;
    delete j.importMapBaseURL;
  }

  assert_no_extra_properties(
      j,
      ['expectedResults', 'baseURL', 'name', 'parsedImportMap',
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
    for (const key of
        ['expectedResults', 'parsedImportMap', 'baseURL', 'name']) {
      assert_own_property(j, key, j.name);
    }

    for (const specifier in j.expectedResults) {
      const expected = j.expectedResults[specifier];
      promise_test(async t => {
          if (expected === null) {
            return promise_rejects(t, new TypeError(),
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
}

export async function runTestsFromJSON(jsonURL) {
  const response = await fetch(jsonURL);
  const json = await response.json();
  await runTests(json);
}
