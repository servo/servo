// META: script=/common/get-host-info.sub.js
// META: script=/common/utils.js
// META: script=/common/dispatcher/dispatcher.js
// META: script=/speculation-rules/prefetch/resources/utils.sub.js
// META: script=resources/utils.js
// META: timeout=long

const port = get_host_info().HTTPS_PORT_ELIDED;
const SUCCESS = true;
const FAILURE = false;

function prefetch_test(origin, expectation) {
  promise_test(
      async t => {
        const key = token();
        const value = 'hello';
        const params = new URLSearchParams();
        params.set('key', key);
        params.set('value', value);

        const url = `${origin}${STORE_URL}?${params.toString()}`;
        insertSpeculationRules({
          prefetch: [{source: 'list', urls: [url], eagerness: 'immediate'}]
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
      `Prefetch to ${origin} ${
          expectation === SUCCESS ? 'succeeds' : 'fails'}.`);
}

const test_cases = [
  // Page loads originate from `http://hosts[][]`; prefetching this origin
  // succeeds, while subdomains fail.
  {origin: 'https://{{hosts[][]}}' + port, expectation: SUCCESS},
  {origin: 'https://{{hosts[][www]}}' + port, expectation: FAILURE},
  {origin: 'https://{{hosts[][www1]}}' + port, expectation: FAILURE},
  {origin: 'https://{{hosts[][www2]}}' + port, expectation: FAILURE},
  {origin: 'https://{{hosts[][天気の良い日]}}' + port, expectation: FAILURE},
  {origin: 'https://{{hosts[][élève]}}' + port, expectation: FAILURE},

  // The pattern we've specified in the header ("*://*.hosts[alt]:*/") will
  // match any subdomain of `hosts[alt]` (though not, as it turns out,
  // `hosts[alt]` itself.
  {origin: 'https://{{hosts[alt][]}}' + port, expectation: FAILURE},
  {origin: 'https://{{hosts[alt][www]}}' + port, expectation: SUCCESS},
  {origin: 'https://{{hosts[alt][www1]}}' + port, expectation: SUCCESS},
  {origin: 'https://{{hosts[alt][www2]}}' + port, expectation: SUCCESS},
  {origin: 'https://{{hosts[alt][天気の良い日]}}' + port, expectation: SUCCESS},
  {origin: 'https://{{hosts[alt][élève]}}' + port, expectation: SUCCESS},
];

for (let i = 0; i < test_cases.length; i++) {
  prefetch_test(test_cases[i].origin, test_cases[i].expectation);
}
