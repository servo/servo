// META: script=helpers.js
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
'use strict';

(async function() {
  // This is cross-domain from the current document.
  const wwwAlt = "https://{{hosts[alt][www]}}:{{ports[https][0]}}";

  if (window === window.top) {
    // Test the interaction between two (same-origin) iframes.
    promise_test(async (t) => {
      const responder_html = `${wwwAlt}/storage-access-api/resources/script-with-cookie-header.py?script=embedded_responder.js`;
      const [frame1, frame2] = await Promise.all([
        CreateFrame(responder_html),
        CreateFrame(responder_html),
      ]);

      t.add_cleanup(async () => {
        await SetPermissionInFrame(frame1, [{ name: 'storage-access' }, 'prompt']);
      });

      const observed = ObservePermissionChange(frame2);
      await SetPermissionInFrame(frame1, [{ name: 'storage-access' }, 'granted']);

      const state = await observed;
      assert_equals(state, "granted");
    }, "Permissions grants are observable across same-origin iframes");

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
    assert_equals(permission.state, "denied");

    await test_driver.set_permission({ name: 'storage-access' }, 'prompt');
  }, "Permission denied state can be queried");

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
