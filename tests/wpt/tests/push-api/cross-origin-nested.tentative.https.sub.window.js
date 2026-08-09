// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/notifications/resources/helpers.js

// The syntax below will give us a third party URL.
// See: https://web-platform-tests.org/writing-tests/server-features.html
const thirdPartyIframe =
  'https://{{hosts[alt][]}}:{{ports[https][0]}}/push-api/resources/cross-origin-nested-parent.sub.html';
const promises = new Map();

// Firefox and Chrome deny notification permission in a third party partitioned
// iframe even if the permission is granted for origin of the iframe.

// Set up the listeners and then create a third party iframe.
// The iframe will again create a first party iframe.
promise_setup(async () => {
  await trySettingPermission("granted");

  // parent: the third party iframe
  // child: the first party iframe in the third party one (ABA)
  for (const iframe of ["parent", "child"]) {
    // from the iframe window, or the worker opened from there
    for (const worker of ["", "Worker"]) {
      const sender = iframe + worker;
      promises.set(sender, new Promise(r => window.addEventListener("message", ev => {
        if (ev.data.sender === sender) {
          r(ev.data);
        }
      })));
    }
  }

  const iframe = document.createElement("iframe");
  iframe.src = thirdPartyIframe;
  document.body.append(iframe);
})

promise_test(async t => {
  const result = await promises.get("parent");
  assert_false(result.subscribed, `subscription should not happen`);
}, "third party iframe");

promise_test(async t => {
  const result = await promises.get("child");
  assert_false(result.subscribed, `subscription should not happen`);
}, "nested first party iframe");

promise_test(async t => {
  const result = await promises.get("parentWorker");
  assert_false(result.subscribed, `subscription should not happen`);
}, "third party worker");

promise_test(async t => {
  const result = await promises.get("childWorker");
  assert_false(result.subscribed, `subscription should not happen`);
}, "nested first party worker");
