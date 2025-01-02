const executor_path = '/common/dispatcher/executor.html?pipe=';
const coop_header = policy => {
  return `|header(Cross-Origin-Opener-Policy,${policy})`;
};

function getExecutorPath(uuid, origin, coop_header) {
  return origin.origin + executor_path + coop_header + `&uuid=${uuid}`;
}

// Open a same-origin popup with `opener_coop` header, then open a popup with
// `openee_coop` header. Check whether the opener can script the openee.
const test_noopener_opening_popup = (
  opener_coop,
  openee_coop,
  origin,
  opener_expectation
) => {
  promise_test(async t => {
    // Set up dispatcher communications.
    const popup_token = token();
    const reply_token = token();
    const popup_reply_token = token();
    const popup_openee_token = token();

    const popup_url = getExecutorPath(
      popup_token, SAME_ORIGIN, coop_header(opener_coop));

    // We open a popup and then ping it, it will respond after loading.
    const popup = window.open(popup_url);
    t.add_cleanup(() => send(popup_token, 'window.close()'));
    send(popup_token, `send('${reply_token}', 'Popup loaded');`);
    assert_equals(await receive(reply_token), 'Popup loaded');

    if (opener_coop == 'noopener-allow-popups') {
      // Assert that we can't script the popup.
      await t.step_wait(() => popup.closed, 'Opener popup.closed')
    }

    // Ensure that the popup has no access to its opener.
    send(popup_token, `
      let openerDOMAccessAllowed = false;
      try {
        openerDOMAccessAllowed = !!self.opener.document.URL;
      } catch(ex) {}
      const payload = {
        opener: !!self.opener,
        openerDOMAccess: openerDOMAccessAllowed
      };
      send('${reply_token}', JSON.stringify(payload));
    `);
    let payload = JSON.parse(await receive(reply_token));
    if (opener_coop == 'noopener-allow-popups') {
      assert_false(payload.opener, 'popup opener');
      assert_false(payload.openerDOMAccess, 'popup DOM access');
    }

    // Open another popup from inside the popup, and wait for it to load.
    const popup_openee_url = getExecutorPath(popup_openee_token, origin,
      coop_header(openee_coop));
    send(popup_token, `
      window.openee = open("${popup_openee_url}");
      await receive('${popup_reply_token}');
      const payload = {
        openee: !!window.openee,
        closed: window.openee.closed
      };
      send('${reply_token}', JSON.stringify(payload));
    `);
    t.add_cleanup(() => send(popup_token, 'window.openee.close()'));
    // Notify the popup that its openee was loaded.
    send(popup_openee_token, `
      send('${popup_reply_token}', 'popup openee opened');
    `);

    // Assert that the popup has access to its openee.
    payload = JSON.parse(await receive(reply_token));
    assert_true(payload.openee, 'popup openee');

    // Assert that the openee has access to its popup opener.
    send(popup_openee_token, `
      let openerDOMAccessAllowed = false;
      try {
        openerDOMAccessAllowed = !!self.opener.document.URL;
      } catch(ex) {}
      const payload = {
        opener: !!self.opener,
        openerDOMAccess: openerDOMAccessAllowed
      };
      send('${reply_token}', JSON.stringify(payload));
    `);
    payload = JSON.parse(await receive(reply_token));
    if (opener_expectation) {
      assert_true(payload.opener, 'Opener is not null');
      assert_true(payload.openerDOMAccess, 'No DOM access');
    } else {
      assert_false(payload.opener, 'Opener is null');
      assert_false(payload.openerDOMAccess, 'No DOM access');
    }
  },
  'noopener-allow-popups ensures that the opener cannot script the openee,' +
  ' but further popups with no COOP can access their opener: ' +
  opener_coop + '/' + openee_coop + ':' + origin == SAME_ORIGIN);
};

// Open a same-origin popup with `popup_coop` header, then navigate away toward
// one with `noopener-allow-popups`. Check the opener can't script the openee.
const test_noopener_navigating_away = (popup_coop) => {
  promise_test(async t => {
    // Set up dispatcher communications.
    const popup_token = token();
    const reply_token = token();
    const popup_reply_token = token();
    const popup_second_token = token();

    const popup_url =
      getExecutorPath(popup_token, SAME_ORIGIN, coop_header(popup_coop));

    // We open a popup and then ping it, it will respond after loading.
    const popup = window.open(popup_url);
    send(popup_token, `send('${reply_token}', 'Popup loaded');`);
    assert_equals(await receive(reply_token), 'Popup loaded');
    t.add_cleanup(() => send(popup_token, 'window.close()'));

    // Assert that we can script the popup.
    assert_not_equals(popup.window, null, 'can script the popup');
    assert_false(popup.closed, 'popup closed');

    // Ensure that the popup has no access to its opener.
    send(popup_token, `
      let openerDOMAccessAllowed = false;
      try {
        openerDOMAccessAllowed = !!self.opener.document.URL;
      } catch(ex) {}
      const payload = {
        opener: !!self.opener,
        openerDOMAccess: openerDOMAccessAllowed
      };
      send('${reply_token}', JSON.stringify(payload));
    `);
    let payload = JSON.parse(await receive(reply_token));
    assert_true(payload.opener, 'popup opener');
    assert_true(payload.openerDOMAccess, 'popup DOM access');

    // Navigate the popup.
    const popup_second_url = getExecutorPath(
      popup_second_token, SAME_ORIGIN,
      coop_header('noopener-allow-popups'));

    send(popup_token, `
      window.location = '${popup_second_url}';
    `);

    // Notify the popup that its openee was loaded.
    send(popup_second_token, `
      send('${reply_token}', 'popup navigated away');
    `);
    assert_equals(await receive(reply_token), 'popup navigated away');

    assert_equals(popup.window, null);
    assert_true(popup.closed, 'popup.closed');
  },
  'noopener-allow-popups ensures that the opener cannot script the openee,' +
  ' even after a navigation: ' + popup_coop);
};
