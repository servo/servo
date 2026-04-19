// META: script=/common/get-host-info.sub.js
// META: script=/common/utils.js
// META: script=resources/utils.js
// META: timeout=long
//
// Tests that <link rel="stylesheet"> to cross-origin URLs is blocked by
// Connection-Allowlist: (response-origin). Stylesheets are subresource
// fetches and should be subject to the same fetch restrictions as scripts,
// images, and other subresources.

const port = get_host_info().HTTP_PORT_ELIDED;
const SUCCESS = true;
const FAILURE = false;

function stylesheet_test(origin, expectation) {
  promise_test(
      async t => {
        const key = token();
        const value = 'hello';
        const params = new URLSearchParams();
        params.set('key', key);
        params.set('value', value);

        const link = document.createElement('link');
        link.rel = 'stylesheet';
        link.href = `${origin}${STORE_URL}?${params.toString()}`;
        document.head.appendChild(link);
        t.add_cleanup(() => link.remove());

        if (expectation === SUCCESS) {
          const result = await nextValueFromServer(key);
          assert_equals(result, value);
        } else {
          const result = await Promise.race([
            new Promise(r => t.step_timeout(r, 1000)),
            nextValueFromServer(key)
          ]);
          assert_true(typeof result === 'undefined');
        }
      },
      `Stylesheet to ${origin} ${
          expectation === SUCCESS ? 'succeeds' : 'fails'}.`);
}

const test_cases = [
  // Page loads from http://hosts[][]; only that origin is allowlisted.
  {origin: 'http://{{hosts[][]}}' + port, expectation: SUCCESS},
  {origin: 'http://{{hosts[][www]}}' + port, expectation: FAILURE},
  {origin: 'http://{{hosts[][www1]}}' + port, expectation: FAILURE},
  {origin: 'http://{{hosts[alt][]}}' + port, expectation: FAILURE},
  {origin: 'http://{{hosts[alt][www]}}' + port, expectation: FAILURE},
];

for (let i = 0; i < test_cases.length; i++) {
  stylesheet_test(test_cases[i].origin, test_cases[i].expectation);
}
