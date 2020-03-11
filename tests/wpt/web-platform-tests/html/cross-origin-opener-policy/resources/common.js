const SAME_ORIGIN = {origin: get_host_info().HTTPS_ORIGIN, name: "SAME_ORIGIN"};
const SAME_SITE = {origin: get_host_info().HTTPS_REMOTE_ORIGIN, name: "SAME_SITE"};
const CROSS_ORIGIN = {origin: get_host_info().HTTPS_NOTSAMESITE_ORIGIN, name: "CROSS_ORIGIN"}

function verify_window(callback, w, hasOpener) {
  // If there's no opener, the w must be closed:
  assert_equals(w.closed, !hasOpener, 'w.closed');
  // Opener's access on w.length is possible only if hasOpener:
  assert_equals(w.length, hasOpener? 1: 0, 'w.length');
  callback();
}

function validate_results(callback, test, w, channelName, hasOpener, openerDOMAccess, payload) {
  assert_equals(payload.name, hasOpener ? channelName : "", 'name');
  assert_equals(payload.opener, hasOpener, 'opener');
  // TODO(zcorpan): add openerDOMAccess expectations to all tests
  if (openerDOMAccess !== undefined) {
    assert_equals(payload.openerDOMAccess, openerDOMAccess, 'openerDOMAccess');
  }

  // The window proxy in Chromium might still reflect the previous frame,
  // until its unloaded. This delays the verification of w here.
  if( !w.closed && w.length == 0) {
    test.step_timeout( () => {
        verify_window(callback, w, hasOpener);
    }, 500);
  } else {
    verify_window(callback, w, hasOpener);
  }
}

function url_test(t, url, channelName, hasOpener, openerDOMAccess) {
  const bc = new BroadcastChannel(channelName);
  bc.onmessage = t.step_func(event => {
    const payload = event.data;
    validate_results(() => { t.done(); }, t, w, channelName, hasOpener, openerDOMAccess, payload);
  });

  const w = window.open(url, channelName);

  // Close the popup once the test is complete.
  // The browsing context might be closed hence use the broadcast channel
  // to trigger the closure.
  t.add_cleanup(() => {
    bc.postMessage("close");
  });
}

function coop_coep_test(t, host, coop, coep, channelName, hasOpener, openerDOMAccess) {
  url_test(t, `${host.origin}/html/cross-origin-opener-policy/resources/coop-coep.py?coop=${encodeURIComponent(coop)}&coep=${coep}&channel=${channelName}`, channelName, hasOpener, openerDOMAccess);
}

function coop_test(t, host, coop, channelName, hasOpener) {
  coop_coep_test(t, host, coop, "", channelName, hasOpener);
}

function run_coop_tests(documentCOOPValueTitle, testArray) {
  for (const test of testArray) {
    async_test(t => {
      coop_test(t, test[0], test[1],
                `${documentCOOPValueTitle}_to_${test[0].name}_${test[1].replace(/ /g,"-")}`,
                test[2]);
    }, `${documentCOOPValueTitle} document opening popup to ${test[0].origin} with COOP: "${test[1]}"`);
  }
}

function run_coop_test_iframe (documentTitle, iframe_origin, popup_origin, popup_coop, expects_opener, expects_name) {
  const name = iframe_origin.name + "_iframe_opening_" + popup_origin.name + "_popup_with_coop_" + popup_coop;
  async_test(t => {
      const frame = document.createElement("iframe");

      // Close the popup and remove the frame once the test is
      // complete. The browsing context might be closed hence use the
      // broadcast channel to trigger the closure.
      t.add_cleanup(() => {
        frame.remove();
        bc.postMessage("close");
      });

      const origin = CROSS_ORIGIN.origin;
      const path = new URL("resources/iframe-popup.sub.html", window.location).pathname;
      const bc = new BroadcastChannel(name);
      frame.src = `${iframe_origin.origin}${path}?popup_origin=${popup_origin.origin}&popup_coop=${popup_coop}&channel=${name}`;

      bc.onmessage = t.step_func_done(event => {
              const payload = event.data;
              assert_equals(payload.opener, expects_opener, 'opener');
              assert_equals(payload.name, expects_name? name:"", 'name');
      });
      document.body.append(frame);
  }, `${documentTitle} with ${iframe_origin.name} iframe opening popup a ${popup_origin.name} with COOP: ${popup_coop}`);
}