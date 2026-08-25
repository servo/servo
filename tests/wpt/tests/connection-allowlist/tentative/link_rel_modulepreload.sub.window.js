// META: script=/common/get-host-info.sub.js
// META: script=/common/utils.js
// META: script=resources/utils.js
// META: timeout=long

const port = get_host_info().HTTP_PORT_ELIDED;
const SUCCESS = true;
const FAILURE = false;

function modulepreload_test(origin, expectation) {
  promise_test(
      async t => {
        const key = token();
        const value = 'hello';
        const params = new URLSearchParams();
        params.set('key', key);
        params.set('value', value);

        const link = document.createElement('link');
        link.rel = 'modulepreload';
        link.href = `${origin}${STORE_URL}?${params.toString()}`;
        document.head.appendChild(link);
        t.add_cleanup(() => link.remove());

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
      `Modulepreload to ${origin} ${
          expectation === SUCCESS ? 'succeeds' : 'fails'}.`);
}

const test_cases = [
  // Page loads originate from `http://hosts[][]`; modulepreloading this origin
  // succeeds, while subdomains fail.
  {origin: 'http://{{hosts[][]}}' + port, expectation: SUCCESS},
  {origin: 'http://{{hosts[][www]}}' + port, expectation: FAILURE},
  {origin: 'http://{{hosts[][www1]}}' + port, expectation: FAILURE},
  {origin: 'http://{{hosts[][www2]}}' + port, expectation: FAILURE},
  {origin: 'http://{{hosts[][天気の良い日]}}' + port, expectation: FAILURE},
  {origin: 'http://{{hosts[][élève]}}' + port, expectation: FAILURE},

  // The pattern we've specified in the header ("*://*.hosts[alt]:*/") will
  // match any subdomain of `hosts[alt]` (though not, as it turns out,
  // `hosts[alt]` itself.
  {origin: 'http://{{hosts[alt][]}}' + port, expectation: FAILURE},
  {origin: 'http://{{hosts[alt][www]}}' + port, expectation: SUCCESS},
  {origin: 'http://{{hosts[alt][www1]}}' + port, expectation: SUCCESS},
  {origin: 'http://{{hosts[alt][www2]}}' + port, expectation: SUCCESS},
  {origin: 'http://{{hosts[alt][天気の良い日]}}' + port, expectation: SUCCESS},
  {origin: 'http://{{hosts[alt][élève]}}' + port, expectation: SUCCESS},
];

for (let i = 0; i < test_cases.length; i++) {
  modulepreload_test(test_cases[i].origin, test_cases[i].expectation);
}
