// META: script=/common/get-host-info.sub.js
//
// The following tests assume the policy `Connection-Allowlist: (response-origin)` has been set.
// Redirects from a connection-allowlisted URL should be blocked by default.

const port = get_host_info().HTTP_PORT_ELIDED;
const SUCCESS = true;
const FAILURE = false;

function redirect_test(origin, target_origin, expectation) {
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

    const target_url = target_origin + "/connection-allowlist/tentative/resources/post-message.html";
    iframe.src = origin + "/common/redirect.py?status=302&location=" + encodeURIComponent(target_url);
    document.body.appendChild(iframe);
    await p;
    document.body.removeChild(iframe);

    if (expectation === SUCCESS) {
      assert_true(received_message, `Redirect from ${origin} to ${target_origin} should have succeeded.`);
    } else {
      assert_false(received_message, `Redirect from ${origin} to ${target_origin} should have failed.`);
    }
  }, `Redirect from ${origin} to ${target_origin} should ${expectation === SUCCESS ? "succeed" : "fail"}.`);
}

// We're loading this page from `http://{{hosts[][]}}`.
// The connection allowlist header is `Connection-Allowlist: (response-origin)`.
// Thus, only `http://{{hosts[][]}}` is allowlisted for navigations.

// Redirect from an allowlisted origin (same-origin):
// origin: http://{{hosts[][]}} (allowed by allowlist)
// target: http://{{hosts[][]}} (also allowed)
// This should FAIL because redirects are default-blocked for allowlisted navigations.
redirect_test("http://{{hosts[][]}}" + port, "http://{{hosts[][]}}" + port, FAILURE);

// Redirect from an allowlisted origin to a different origin:
// origin: http://{{hosts[][]}} (allowed by allowlist)
// target: http://{{hosts[alt][]}} (not allowed)
// This should FAIL.
redirect_test("http://{{hosts[][]}}" + port, "http://{{hosts[alt][]}}" + port, FAILURE);

// Initial navigation to a non-allowlisted origin:
// origin: http://{{hosts[alt][]}} (not allowed)
// This is blocked before the redirect even happens.
redirect_test("http://{{hosts[alt][]}}" + port, "http://{{hosts[][]}}" + port, FAILURE);

