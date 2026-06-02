// To use the functions below, be sure to include the following files in your
// test:
// - "/common/get-host-info.sub.js" to get the different origin values.
// - "common.js" to have the origins easily available.
// - "/common/dispatcher/dispatcher.js" for cross-origin messaging.
// - "/common/utils.js" for token().

function getBaseExecutorPath(origin) {
  return origin + '/common/dispatcher/executor.html';
}

function getHeadersPipe(headers) {
  const coop_header = headers.coop ?
    `|header(Cross-Origin-Opener-Policy,${encodeURIComponent(headers.coop)})` : '';
  const coep_header = headers.coep ?
    `|header(Cross-Origin-Embedder-Policy,${encodeURIComponent(headers.coep)})` : '';
  return coop_header + coep_header;
}

function getExecutorPath(uuid, origin, headers) {
  return getBaseExecutorPath(origin) +
    `?uuid=${uuid}` +
    `&pipe=${getHeadersPipe(headers)}`;
}

function evaluate(target_token, script) {
  const reply_token = token();
  send(target_token, `send('${reply_token}', ${script});`);
  return receive(reply_token);
}

// Return true if an opened iframe can access |property| on a stored
// window.popup object without throwing an error.
function iframeCanAccessProperty(iframe_token, property) {
  const reply_token = token();
  send(iframe_token,
    `try {
       const unused = window.popup['${property}'];
       send('${reply_token}', 'true')
     } catch (errors) {
       send('${reply_token}', 'false')
     }`);
  return receive(reply_token);
}

// Returns the script necessary to open a popup, given the method in
// `popup_via`. Supported methods are 'window_open' that leverages
// window.open(), 'anchor' that creates an <a> HTML element and clicks on it,
// and 'form' that creates a form and submits it.
function popupOpeningScript(popup_via, popup_url, popup_origin, headers,
    popup_token) {
  if (popup_via === 'window_open')
    return `window.popup = window.open('${popup_url}', '${popup_token}');`;

  if (popup_via === 'anchor') {
    return `
      const anchor = document.createElement('a');
      anchor.href = '${popup_url}';
      anchor.rel = "opener";
      anchor.target = '${popup_token}';
      anchor.innerText = "anchor";
      document.body.appendChild(anchor);
      anchor.click();
      `;
  }

  if (popup_via === "form") {
    return `
      const form = document.createElement("form");
      form.action = '${getBaseExecutorPath(popup_origin.origin)}';
      form.target = '${popup_token}';
      form.method = 'GET';
      const add_param = (name, value) => {
        const input = document.createElement("input");
        input.name = name;
        input.value = value;
        form.appendChild(input);
      };
      add_param("uuid", "${popup_token}");
      add_param("pipe", "${getHeadersPipe(headers)}");
      document.body.appendChild(form);
      form.submit();
      `;
  }

  assert_not_reached('Unrecognized popup opening method.');
}

function promise_test_parallel(promise, description) {
  async_test(test => {
    promise(test)
        .then(() => test.done())
        .catch(test.step_func(error => { throw error; }));
  }, description);
};

// Verifies that a popup with origin `popup_origin` and headers `headers` has
// the expected `opener_state` after being opened from an iframe with origin
// `iframe_origin`.
function iframe_test(description, iframe_origin, popup_origin, headers,
    expected_opener_state) {
  for (const popup_via of ['window_open', 'anchor','form']) {
    promise_test_parallel(async t => {
      const iframe_token = token();
      const popup_token = token();
      const reply_token = token();

      const frame = document.createElement("iframe");
      const iframe_url = getExecutorPath(
        iframe_token,
        iframe_origin.origin,
        {});

      frame.src = iframe_url;
      document.body.append(frame);

      send(iframe_token, `send('${reply_token}', 'Iframe loaded');`);
      assert_equals(await receive(reply_token), 'Iframe loaded');

      const popup_url = getExecutorPath(
        popup_token,
        popup_origin.origin,
        headers);

      // We open popup and then ping it, it will respond after loading.
      send(iframe_token, popupOpeningScript(popup_via, popup_url, popup_origin,
                                            headers, popup_token));
      send(popup_token, `send('${reply_token}', 'Popup loaded');`);
      assert_equals(await receive(reply_token), 'Popup loaded');

      // Make sure the popup and the iframe are removed once the test has run,
      // keeping a clean state.
      add_completion_callback(() => {
        frame.remove();
        send(popup_token, `close()`);
      });

      // Give some time for things to settle across processes etc. before
      // proceeding with verifications.
      await new Promise(resolve => { t.step_timeout(resolve, 500); });

      // Verify that the opener is in the state we expect it to be in.
      switch (expected_opener_state) {
        case 'preserved': {
          assert_equals(
            await evaluate(popup_token, 'opener != null'), "true",
            'Popup has an opener?');
          assert_equals(
            await evaluate(popup_token, `name === '${popup_token}'`), "true",
            'Popup has a name?');

          // When the popup was created using window.open, we've kept a handle
          // and we can do extra verifications.
          if (popup_via === 'window_open') {
            assert_equals(
              await evaluate(iframe_token, 'popup != null'), "true",
              'Popup handle is non-null in iframe?');
            assert_equals(
              await evaluate(iframe_token, 'popup.closed'), "false",
              'Popup appears closed from iframe?');
            assert_equals(
              await iframeCanAccessProperty(iframe_token, "document"),
              popup_origin === iframe_origin ? "true" : "false",
              'Iframe has dom access to the popup?');
            assert_equals(
              await iframeCanAccessProperty(iframe_token, "frames"), "true",
              'Iframe has cross origin access to the popup?');
          }
          break;
        }
        case 'restricted': {
          assert_equals(
            await evaluate(popup_token, 'opener != null'), "true",
            'Popup has an opener?');
          assert_equals(
            await evaluate(popup_token, `name === ''`), "true",
            'Popup name is cleared?');

          // When the popup was created using window.open, we've kept a handle
          // and we can do extra verifications.
          if (popup_via === 'window_open') {
            assert_equals(
              await evaluate(iframe_token, 'popup != null'), "true",
              'Popup handle is non-null in iframe?');
            assert_equals(
              await evaluate(iframe_token, 'popup.closed'), "false",
              'Popup appears closed from iframe?');
            assert_equals(
              await iframeCanAccessProperty(iframe_token, "document"), "false",
              'Iframe has dom access to the popup?');
            assert_equals(
              await iframeCanAccessProperty(iframe_token, "frames"), "false",
              'Iframe has cross origin access to the popup?');
            assert_equals(
              await iframeCanAccessProperty(iframe_token, "closed"), "true",
              'Iframe has limited cross origin access to the popup?');
          }
          break;
        }
        case 'severed': {
          assert_equals(await evaluate(popup_token, 'opener != null'), "false",
                       'Popup has an opener?');
          assert_equals(
            await evaluate(popup_token, `name === ''`), "true",
            'Popup name is cleared?');

          // When the popup was created using window.open, we've kept a handle
          // and we can do extra verifications.
          if (popup_via === 'window_open') {
            assert_equals(
              await evaluate(iframe_token, 'popup != null'), "true",
              'Popup handle is non-null in iframe?');
            assert_equals(
              await evaluate(iframe_token, 'popup.closed'), "true",
              'Popup appears closed from iframe?');
          }
          break;
        }
        case 'noopener': {
          assert_equals(await evaluate(popup_token, 'opener != null'), "false",
                        'Popup has an opener?');
          assert_equals(
            await evaluate(popup_token, `name === ''`), "true",
            'Popup name is cleared?');

          // When the popup was created using window.open, we've kept a handle
          // and we can do extra verifications.
          if (popup_via === 'window_open') {
            assert_equals(
              await evaluate(iframe_token, 'popup == null'), "true",
              'Popup handle is null in iframe?');
          }
          break;
        }
        default:
          assert_not_reached('Unrecognized opener state: ' +
            expected_opener_state);
      }
    }, `${description} with ${popup_via}`);
  }
}
