// META: script=/common/get-host-info.sub.js
//
// The following tests assume the policy `Connection-Allowlist: (response-origin "*://:subdomain.{{hosts[alt][]}}:*")` has been set.

const port = get_host_info().HTTP_PORT_ELIDED;
const SUCCESS = true;
const FAILURE = false;

function navigation_test(origin, expectation) {
  promise_test(async t => {
    const iframe = document.createElement("iframe");
    let received_message = false;
    const handler = (e) => {
      if (e.data === "loaded") {
        received_message = true;
      }
    };
    window.addEventListener("message", handler);
    t.add_cleanup(() => window.removeEventListener("message", handler));

    const p = new Promise((resolve) => {
      iframe.onload = () => {
        // If onload fires, it might be the success page or an error page.
        // We wait a short bit to ensure any postMessage has time to arrive.
        step_timeout(() => resolve(), 50);
      };
      iframe.onerror = () => resolve();
    });

    iframe.src = origin + "/connection-allowlist/tentative/resources/post-message.html";
    document.body.appendChild(iframe);
    await p;
    document.body.removeChild(iframe);

    if (expectation === SUCCESS) {
      assert_true(received_message, `Navigation to ${origin} should have succeeded.`);
    } else {
      assert_false(received_message, `Navigation to ${origin} should have failed.`);
    }
  }, `Navigation to ${origin} should ${expectation === SUCCESS ? "succeed" : "fail"}.`);
}

const test_cases = [
  // We're loading this page from `http://hosts[][]`, so that origin should
  // succeed, while its subdomains should fail.
  { origin: "http://{{hosts[][]}}" + port, expectation: SUCCESS },
  { origin: "http://{{hosts[][www]}}" + port, expectation: FAILURE },
  { origin: "http://{{hosts[][www1]}}" + port, expectation: FAILURE },
  { origin: "http://{{hosts[][www2]}}" + port, expectation: FAILURE },
  { origin: "http://{{hosts[][天気の良い日]}}" + port, expectation: FAILURE },
  { origin: "http://{{hosts[][élève]}}" + port, expectation: FAILURE },

  // The pattern we've specified in the header
  // ("*://:subdomain.{{hosts[alt][]}}:*") will match any subdomain of
  // `hosts[alt]` (though not, as it turns out, `hosts[alt]` itself.
  { origin: "http://{{hosts[alt][]}}" + port, expectation: FAILURE },
  { origin: "http://{{hosts[alt][www]}}" + port, expectation: SUCCESS },
  { origin: "http://{{hosts[alt][www1]}}" + port, expectation: SUCCESS },
  { origin: "http://{{hosts[alt][www2]}}" + port, expectation: SUCCESS },
  { origin: "http://{{hosts[alt][天気の良い日]}}" + port, expectation: SUCCESS },
  { origin: "http://{{hosts[alt][élève]}}" + port, expectation: SUCCESS },
];

for (let i = 0; i < test_cases.length; i++) {
  navigation_test(test_cases[i].origin, test_cases[i].expectation);
}
