ECHO_URL = "resources/echo-critical-hint.py"
REDIRECT_URL = "resources/redirect-critical-hint.py"
CLEAR_URL = "resources/clear-client-hints.html"

// Accept-CH preferences are persisted per-origin and outlive a single test.
// Since every test in this directory shares the same origin (and, in most
// runners, the same browsing session), hints persisted by an earlier test
// would otherwise be sent on the first request of a later test, which breaks
// tests expecting a request *without* hints (e.g. the mis-matched,
// subresource, and iframe tests). Fetching CLEAR_URL returns a
// `Clear-Site-Data: "clientHints"` header which resets that state.
clear_client_hints = () => fetch(CLEAR_URL);

message_listener = (t, message) =>
  (e) => {
    t.step(()=>{assert_equals(e.data, message)});
    t.done();
  }

// Serialize message tests within the same file (e.g. mis-matched-count and
// non-secure) so that concurrent popups do not race on window 'message' events
// or on persisted Accept-CH state.
let message_test_queue = Promise.resolve();

make_message_test = (url, message) =>
  (t) => {
    message_test_queue = message_test_queue.then(() => new Promise((resolve) => {
      t.add_cleanup(resolve);
      clear_client_hints().then(t.step_func(() => {
        const popup_window = window.open("/common/blank.html");
        assert_not_equals(popup_window, null, "Popup windows not allowed?");
        t.add_cleanup(() => popup_window.close());
        window.addEventListener('message', message_listener(t, message), {once: true});
        popup_window.location = url;
      }));
    }));
  }
