// To use the functions below, be sure to include the following files in your
// test:
// - "/common/get-host-info.sub.js" to get the different origin values.
// - "common.js" to have the origins easily available.
// - "/common/dispatcher/dispatcher.js" for cross-origin messaging.
// - "/common/utils.js" for token().

function getExecutorPath(uuid, origin, headers) {
  const executor_path = '/common/dispatcher/executor.html?';
  const coop_header = headers.coop ?
    `|header(Cross-Origin-Opener-Policy,${encodeURIComponent(headers.coop)})` : '';
  const coep_header = headers.coep ?
    `|header(Cross-Origin-Embedder-Policy,${encodeURIComponent(headers.coep)})` : '';
  return origin +
         executor_path +
         `uuid=${uuid}` +
         '&pipe=' + coop_header + coep_header;
}

function getPopupHasOpener(popup_token) {
  const reply_token = token();
  send(popup_token, `send('${reply_token}', window.opener != null);`);
  return receive(reply_token);
}

// Return true if |object|.|property| can be called without throwing an error.
function canAccessProperty(object, property) {
  try {
    const unused = object[property];
    return true;
  } catch (errors) {
    return false;
  }
}

// Verifies that a popup with origin `origin` and headers `headers` has
// the expected `opener_state` after being opened.
async function popup_test(description, origin, headers, expected_opener_state) {
  promise_test(async t => {
    const popup_token = token();
    const reply_token = token();

    const popup_url = getExecutorPath(
      popup_token,
      origin.origin,
      headers);

    // We open popup and then ping it, it will respond after loading.
    const popup = window.open(popup_url);
    send(popup_token, `send('${reply_token}', 'Popup loaded');`);
    assert_equals(await receive(reply_token), 'Popup loaded');

    // Make sure the popup will be closed once the test has run, keeping a clean
    // state.
    t.add_cleanup(() => {
      send(popup_token, `close()`);
    });

    // Give some time for things to settle across processes etc. before
    // proceeding with verifications.
    await new Promise(resolve => { t.step_timeout(resolve, 500); });

    // Verify that the opener is in the state we expect it to be in.
    switch (expected_opener_state) {
      case 'preserved': {
        assert_false(popup.closed, 'Popup is closed from opener?');
        assert_true(await getPopupHasOpener(popup_token) === "true",
                    'Popup has nulled opener?');
        assert_equals(canAccessProperty(popup, "document"),
                      origin === SAME_ORIGIN,
                      'Main page has dom access to the popup?');
        assert_true(canAccessProperty(popup, "frames"),
                    'Main page has cross origin access to the popup?');
        break;
      }
      case 'restricted': {
        assert_false(popup.closed, 'Popup is closed from opener?');
        assert_true(await getPopupHasOpener(popup_token) === "true",
                    'Popup has nulled opener?');
        assert_false(canAccessProperty(popup, "document"),
                     'Main page has dom access to the popup?');
        assert_false(canAccessProperty(popup, "frames"),
                    'Main page has cross origin access to the popup?');
        assert_true(canAccessProperty(popup, "closed"),
                    'Main page has limited cross origin access to the popup?');
        break;
      }
      case 'severed': {
        assert_true(popup.closed, 'Popup is closed from opener?');
        assert_false(await getPopupHasOpener(popup_token) === "true",
                     'Popup has nulled opener?');
        break;
      }
      default:
        assert_unreached(true, "Unrecognized opener relationship: " + expected_opener_state);
    }
  }, description);
}

