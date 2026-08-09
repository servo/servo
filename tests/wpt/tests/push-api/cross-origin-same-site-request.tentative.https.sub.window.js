// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/notifications/resources/helpers.js

// The syntax below will give us a same-site cross-origin URL.
// See: https://web-platform-tests.org/writing-tests/server-features.html
const sameSiteIframe =
  'https://{{hosts[][www1]}}:{{ports[https][0]}}/push-api/resources/cross-origin-nested-child.sub.html';
let promise;

// Firefox and Chrome deny notification permission in a same-site cross-origin
// iframe even if the permission is granted for origin of the iframe.

// Set up the listeners and then create a same-site iframe.
promise_setup(async () => {
  await trySettingPermission("granted");

  promise = new Promise(r => window.addEventListener("message", ev => {
    if (ev.data.sender === "child") {
      r(ev.data);
    }
  }));

  const iframe = document.createElement("iframe");
  iframe.src = sameSiteIframe;
  document.body.append(iframe);
})

promise_test(async t => {
  const result = await promise;
  assert_false(result.subscribed, `subscription should not happen`);
}, "same-site cross-origin iframe");
