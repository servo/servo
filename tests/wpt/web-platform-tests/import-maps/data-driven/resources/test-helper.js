setup({allow_uncaught_exception : true});

// Set window.useInternalMethods = true when needed && available.

let registration;
const scope = './scope/';

// Global setup: this must be the first promise_test.
promise_test(async (t) => {
  const script = 'service-worker.js';

  registration =
      await service_worker_unregister_and_register(t, script, scope);
  window.worker = registration.installing;
  await wait_for_state(t, window.worker, 'activated');
}, 'global setup');

export function setupGlobalCleanup() {
  // Global cleanup: the final promise_test.
  promise_test(() => {
     return registration.unregister();
  }, 'global cleanup');
}

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
      <body>
      <script src="${location.origin}/import-maps/data-driven/resources/test-helper-iframe.js"></script>
      <script type="importmap" onerror="onScriptError(event)">
      ${importMapString}
      </script>
      <script type="module">
        if (!window.registrationResult) {
          window.registrationResult = {type: 'Success'};
        }
        window.removeEventListener('error', window.windowErrorHandler);
        parent.postMessage(window.registrationResult, '*');
      </script>
      </body>
    `;

    if (new URL(importMapBaseURL).protocol === 'data:') {
      if (!window.useInternalMethods) {
        throw new Error(
            'Import maps with base URL = data: URL requires internal methods');
      }
      iframe.src = 'data:text/html;base64,' + btoa(testHTML);
    } else {
      // Set the src to `scope` in order to make requests from `iframe`
      // intercepted by the service worker.
      iframe.src = scope;
      iframe.onload = () => {
        iframe.contentDocument.write(
            `<base href="${importMapBaseURL}">` + testHTML);
        iframe.contentDocument.close();
      };
    }
    document.body.appendChild(iframe);
  });
}

// Returns a promise that is resolved with the resulting URL.
// `expectedURL` is a string, or null if to be thrown.
function resolve(specifier, parsedImportMap, baseURL, expectedURL) {
  return new Promise((resolve, reject) => {
    window.addEventListener('message', event => {
        if (event.data.type === 'ResolutionSuccess') {
          resolve(event.data.result);
        } else if (event.data.type === 'Failure') {
          if (event.data.result === 'TypeError') {
            reject(new TypeError(event.data.message));
          } else {
            reject(new Error(event.data.message));
          }
        } else {
          assert_unreached('Invalid message: ' + event.data.type);
        }
      },
      {once: true});

    parsedImportMap.contentWindow.postMessage({action: 'prepareResolve'}, '*');

    navigator.serviceWorker.addEventListener('message', event => {
      // Step 2. After postMessage() at Step 1 is processed, the service worker
      // sends back a message and the parent Window receives the message here
      // and sends a 'resolve' message to the iframe.
      parsedImportMap.contentWindow.postMessage(
          {action: 'resolve',
           specifier: specifier,
           baseURL: baseURL,
           expectedURL: expectedURL,
           useInternalMethods: window.useInternalMethods},
          '*');
    }, {once: true});
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
        {action: 'getParsedImportMap',
         useInternalMethods: window.useInternalMethods}, '*');
  });
}

function assert_no_extra_properties(object, expectedProperties, description) {
  for (const actualProperty in object) {
    assert_true(expectedProperties.indexOf(actualProperty) !== -1,
        description + ': unexpected property ' + actualProperty);
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
          'Success',
          'Import map registration should be successful for resolution tests');
      for (const specifier in j.expectedResults) {
        const expected = j.expectedResults[specifier];
        promise_test(async t => {
            if (expected === null) {
              return promise_rejects_js(t, TypeError,
                  resolve(specifier, j.parsedImportMap, j.baseURL, null));
            } else {
              // Should be resolved to `expected`.
              const actual = await resolve(
                  specifier, j.parsedImportMap, j.baseURL, expected);
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
          assert_equals(j.parsedImportMap.parseImportMapResult, 'ParseError');
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
