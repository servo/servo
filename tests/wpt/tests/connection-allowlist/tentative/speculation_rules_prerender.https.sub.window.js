// META: script=/common/get-host-info.sub.js
// META: script=/common/utils.js
// META: script=/common/dispatcher/dispatcher.js
// META: script=/speculation-rules/prefetch/resources/utils.sub.js
// META: script=resources/utils.js
// META: timeout=long

const port = get_host_info().HTTPS_PORT_ELIDED;
const SUCCESS = true;
const FAILURE = false;

function prerender_test(origin, expectation) {
  promise_test(
      async t => {
        const key = token();
        const value = 'hello';
        const params = new URLSearchParams();
        params.set('key', key);
        params.set('value', value);

        const url = `${origin}${STORE_URL}?${params.toString()}`;
        insertSpeculationRules({
          prerender: [{source: 'list', urls: [url], eagerness: 'immediate'}]
        });

        if (expectation === SUCCESS) {
          const result = await nextValueFromServer(key);
          assert_equals(result, value);
        } else {
          const result = await Promise.race([
            new Promise(r => t.step_timeout(r, 1000)), nextValueFromServer(key)
          ]);
          assert_true(typeof result === 'undefined');
        }
      },
      `Prerender to ${origin} ${
          expectation === SUCCESS ? 'succeeds' : 'fails'}.`);
}

// Note: SpeculationRules API currently does not support cross-site prerender.
// Only same-site prerender is tested. See crbug.com/1176054.
const test_cases = [
  // Page loads originate from `http://hosts[][]`; prerendering this origin
  // succeeds, while subdomains fail except `https://{{hosts[][www1]}}` which is
  // allowed by the allowlist.
  {origin: 'https://{{hosts[][]}}' + port, expectation: SUCCESS},
  {origin: 'https://{{hosts[][www]}}' + port, expectation: FAILURE},
  {origin: 'https://{{hosts[][www1]}}' + port, expectation: SUCCESS},
  {origin: 'https://{{hosts[][www2]}}' + port, expectation: FAILURE},
  {origin: 'https://{{hosts[][天気の良い日]}}' + port, expectation: FAILURE},
  {origin: 'https://{{hosts[][élève]}}' + port, expectation: FAILURE},
];

for (let i = 0; i < test_cases.length; i++) {
  prerender_test(test_cases[i].origin, test_cases[i].expectation);
}
