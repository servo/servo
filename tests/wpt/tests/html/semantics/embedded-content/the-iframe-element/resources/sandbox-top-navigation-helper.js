// To use this file, use the following imports:
// // META: script=/common/dispatcher/dispatcher.js
// // META: script=/common/get-host-info.sub.js
// // META: script=/common/utils.js
// // META: script=/resources/testdriver.js
// // META: script=/resources/testdriver-vendor.js
// // META: script=/html/browsers/browsing-the-web/remote-context-helper/resources/remote-context-helper.js
// // META: script=./resources/sandbox-top-navigation-helper.js

// Helper file that provides various functions to test top-level navigation
// with various frame and sandbox flag configurations.

async function createNestedIframe(parent, origin, frame_sandbox, header_sandbox)
{
  let headers = [];
  if (header_sandbox) {
    headers.push([
      "Content-Security-Policy",
      "sandbox allow-scripts " + header_sandbox
    ]);
  }
  let iframe_attributes = {};
  if (frame_sandbox) {
    iframe_attributes.sandbox = "allow-scripts " + frame_sandbox;
  }
  return parent.addIframe({
    origin: origin,
    scripts: [
      '/resources/testdriver.js',
      '/resources/testdriver-driver.js',
      '/resources/testdriver-vendor.js'
    ],
    headers: headers,
  }, iframe_attributes);
}

async function attemptTopNavigation(iframe, should_succeed) {
  let did_succeed;
  try {
    await iframe.executeScript(() => {
      window.top.location.href = "https://google.com";
    });
    did_succeed = true;
  } catch (e) {
    did_succeed = false;
  }

  assert_equals(did_succeed, should_succeed,
      should_succeed ?
          "The navigation should succeed." :
          "The navigation should fail.");
}

async function setupTest() {
  const rcHelper = new RemoteContextHelper();
  return rcHelper.addWindow(/*config=*/ null, /*options=*/ {});
}

async function activate(iframe) {
  return iframe.executeScript(async () => {
    let b = document.createElement("button");
    document.body.appendChild(b);

    // Since test_driver.bless() does not play nicely with the remote context
    // helper, this is a workaround to trigger user activation in the iframe.
    // This adds a button to the iframe and then simulates hitting the 'tab' key
    // twice. Once to focus on the button, and once to trigger user activation
    // in the iframe (user activation is given to the frame that has focus when
    // the tab key is pressed, not the frame that ends up getting focus). Note
    // that this will result in both the parent and this frame getting user
    // activation. Note that this currently only works for iframes nested 1
    // level deep.
    test_driver.set_test_context(window.top);
    return test_driver.send_keys(document.body, "\uE004\uE004");
  });
}
