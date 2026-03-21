// META: script=/common/get-host-info.sub.js
// META: script=/common/utils.js
//
// The following tests assume the policy `Connection-Allowlist: (response-origin)` has been set.

const port = get_host_info().HTTP_PORT_ELIDED;
const SUCCESS = true;
const FAILURE = false;

function navigation_test(origin, expectation) {
  promise_test(async t => {
    let received_message = false;
    const window_name = token();

    const handler = (e) => {
      if (e.data === "loaded") {
        received_message = true;
        e.source.close();
      }
    };
    window.addEventListener("message", handler);
    t.add_cleanup(() => {
      window.removeEventListener("message", handler);
      // Try to close it just in case it opened but didn't send a message.
      const win = window.open("", window_name);
      if (win) {
        win.close();
      }
    });

    const a = document.createElement("a");
    a.href = origin + "/connection-allowlist/tentative/resources/post-message-opener.html";
    a.target = window_name;
    a.rel = "opener";
    document.body.appendChild(a);
    a.click();

    // Wait for message or timeout.
    // We wait a bit longer because opening a new window can be slow.
    await new Promise(resolve => t.step_timeout(resolve, 1500));

    document.body.removeChild(a);

    if (expectation === SUCCESS) {
      assert_true(received_message, `Navigation via anchor with target=${window_name} to ${origin} should have succeeded.`);
    } else {
      assert_false(received_message, `Navigation via anchor with target=${window_name} to ${origin} should have failed.`);
    }
  }, `Navigation via anchor with target=_blank to ${origin} should ${expectation === SUCCESS ? "succeed" : "fail"}.`);
}

const test_cases = [
  // We're loading this page from `http://hosts[][]`, so that origin should
  // succeed, while its subdomains and cross-site origins should fail:
  { origin: "http://{{hosts[][]}}" + port, expectation: SUCCESS },
  { origin: "http://{{hosts[][www]}}" + port, expectation: FAILURE },
  { origin: "http://{{hosts[alt][]}}" + port, expectation: FAILURE },
];

for (let i = 0; i < test_cases.length; i++) {
  navigation_test(test_cases[i].origin, test_cases[i].expectation);
}
