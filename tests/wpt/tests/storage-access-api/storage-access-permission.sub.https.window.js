// META: script=helpers.js
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
'use strict';

(async function() {
  // These are cross-domain from the current document.
  const wwwAlt = "https://{{hosts[alt][www]}}:{{ports[https][0]}}";
  const www1Alt = "https://{{hosts[alt][www1]}}:{{ports[https][0]}}";
  const responder_html_load_ack = "/storage-access-api/resources/script-with-cookie-header.py?script=embedded_responder.js&should_ack_load=true";

  function permissionChangeEvent() {
    return new Promise(resolve => {
      window.onmessage = e => {
        if (e.data.tag == 'observed_permission_change') {
          resolve(e.data.state);
        }
      };
    });
  }

  // Test the interaction between two (same-origin) iframes.
  promise_test(async (t) => {
    // Note: the web platform doesn't guarantee that each iframe has finished
    // loading (and executing its script) by the time the CreateFrame promise
    // resolves. Therefore the script will signal the parent when it's loaded
    // and safe to proceed. Without this extra synchronization, frames can
    // miss messages that are essential to the test, and cause the test to
    // timeout.
    const frame1_loaded = new Promise(r => {
      onmessage = e => r(e.data);
    });
    const frame1 = await CreateFrame(wwwAlt + responder_html_load_ack);
    assert_equals(await frame1_loaded, "loaded");

    const frame2_loaded = new Promise(r => {
      onmessage = e => r(e.data);
    });
    const frame2 = await CreateFrame(www1Alt + responder_html_load_ack);
    assert_equals(await frame2_loaded, "loaded");

    t.add_cleanup(async () => {
      await SetPermissionInFrame(frame1, [{ name: 'storage-access' }, 'prompt']);
    });

    // Install observer on frame, and wait for acknowledgement that it is
    // installed.
    assert_equals(await ObservePermissionChange(frame2),
                  "permission_change_observer_installed");

    const observed_event = permissionChangeEvent();
    await SetPermissionInFrame(frame1, [{ name: 'storage-access' }, 'granted']);
    const state = await observed_event;
    assert_equals(state, "granted");
  }, 'Permissions grants are observable across same-origin iframes');

  promise_test(async (t) => {
    // Note: the web platform doesn't guarantee that each iframe has finished
    // loading (and executing its script) by the time the CreateFrame promise
    // resolves. Therefore the script will signal the parent when it's loaded
    // and safe to proceed. Without this extra synchronization, frames can
    // miss messages that are essential to the test, and cause the test to
    // timeout.
    const frame1_loaded = new Promise(r => {
      onmessage = e => r(e.data);
    });
    const frame1 = await CreateFrame(wwwAlt + responder_html_load_ack);
    assert_equals(await frame1_loaded, "loaded");

    const frame2_loaded = new Promise(r => {
      onmessage = e => r(e.data);
    });
    const frame2 = await CreateFrame(www1Alt + responder_html_load_ack);
    assert_equals(await frame2_loaded, "loaded");

    t.add_cleanup(async () => {
      await SetPermissionInFrame(frame1, [{ name: 'storage-access' }, 'prompt']);
    });

    // Install observer on frame, and wait for acknowledgement that it is
    // installed.
    assert_equals(await ObservePermissionChange(frame2),
                  "permission_change_observer_installed");

    const observed_event = permissionChangeEvent();
    await SetPermissionInFrame(frame1, [{ name: 'storage-access' }, 'granted']);
    const state = await observed_event;
    assert_equals(state, "granted");
  }, "Permissions grants are observable across same-site iframes");

  promise_test(async () => {
    // Finally run the simple tests below in a separate cross-origin iframe.
    await RunTestsInIFrame('https://{{domains[www]}}:{{ports[https][0]}}/storage-access-api/resources/permissions-iframe.https.html');
  }, "IFrame tests");

})();
