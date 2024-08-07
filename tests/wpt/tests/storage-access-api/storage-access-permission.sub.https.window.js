// META: script=helpers.js
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
'use strict';

(async function() {
  // These are cross-domain from the current document.
  const wwwAlt = "https://{{hosts[alt][www]}}:{{ports[https][0]}}";
  const www1Alt = "https://{{hosts[alt][www1]}}:{{ports[https][0]}}";
  const responder_html_load_ack = "/storage-access-api/resources/script-with-cookie-header.py?script=embedded_responder.js&should_ack_load=true";

  if (window === window.top) {
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

      const observed_event = new Promise(r => {
        onmessage = e => r(e.data);
      });
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

      const observed_event = new Promise(r => {
        onmessage = e => r(e.data);
      });
      await SetPermissionInFrame(frame1, [{ name: 'storage-access' }, 'granted']);
      const state = await observed_event;
      assert_equals(state, "granted");
    }, "Permissions grants are observable across same-site iframes");

    promise_test(async (t) => {
      // Finally run the simple tests below in a separate cross-origin iframe.
      await RunTestsInIFrame('https://{{domains[www]}}:{{ports[https][0]}}/storage-access-api/resources/permissions-iframe.https.html');
    }, "IFrame tests");
    return;
  }

  // We're in a cross-origin, same-site iframe test now.
  test_driver.set_test_context(window.top);

  promise_test(async t => {
    const permission = await navigator.permissions.query({name: "storage-access"});
    assert_equals(permission.name, "storage-access");
    assert_equals(permission.state, "prompt");
  }, "Permission default state can be queried");

  promise_test(async t => {
    t.add_cleanup(async () => {
      await test_driver.set_permission({ name: 'storage-access' }, 'prompt');
    });
    await test_driver.set_permission({ name: 'storage-access' }, 'granted');

    const permission = await navigator.permissions.query({name: "storage-access"});
    assert_equals(permission.name, "storage-access");
    assert_equals(permission.state, "granted");
  }, "Permission granted state can be queried");

  promise_test(async t => {
    t.add_cleanup(async () => {
      await test_driver.set_permission({ name: 'storage-access' }, 'prompt');
    });
    await test_driver.set_permission({ name: 'storage-access' }, 'denied');

    const permission = await navigator.permissions.query({name: "storage-access"});
    assert_equals(permission.name, "storage-access");
    assert_equals(permission.state, "prompt");

    await test_driver.set_permission({ name: 'storage-access' }, 'prompt');
  }, "Permission denied state is hidden");

  promise_test(async t => {
    t.add_cleanup(async () => {
      await test_driver.set_permission({ name: 'storage-access' }, 'prompt');
    });

    const permission = await navigator.permissions.query({name: "storage-access"});

    const p = new Promise(resolve => {
      permission.addEventListener("change", (event) => resolve(event), { once: true });
    });

    await test_driver.set_permission({ name: 'storage-access' }, 'granted');
    await document.requestStorageAccess();

    const event = await p;

    assert_equals(event.target.name, "storage-access");
    assert_equals(event.target.state, "granted");
  }, "Permission state can be observed");
})();
