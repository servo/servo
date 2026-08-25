// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/notifications/resources/helpers.js

// The syntax below will give us a third party URL.
// See: https://web-platform-tests.org/writing-tests/server-features.html
const thirdPartyOrigin = 'https://{{hosts[alt][]}}:{{ports[https][0]}}';
const thirdPartyIframe =
  `${thirdPartyOrigin}/push-api/resources/cross-origin-serviceworker-iframe.sub.html`;
let promise;

promise_setup(async () => {
  await trySettingPermission("granted");

  promise = new Promise(r => window.addEventListener("message", ev => {
    if ("subscribed" in ev.data) {
      r(ev.data)
    }
  }));

  const iframe = document.createElement("iframe");
  iframe.src = thirdPartyIframe;
  document.body.append(iframe);
})

promise_test(async t => {
  const result = await promise;
  assert_false(result.subscribed, `subscription should not happen`);
}, "third party serviceworker");
