// META: timeout=long
// META: script=/common/get-host-info.sub.js
// META: script=/common/utils.js
// META: script=/common/dispatcher/dispatcher.js
// META: script=/html/cross-origin-embedder-policy/credentialless/resources/common.js

const {ORIGIN, REMOTE_ORIGIN} = get_host_info();
const control_iframe = document.createElement('iframe');
const anonymous_iframe = document.createElement('iframe');

promise_setup(async t => {
  const createControlIframe = new Promise(async resolve => {
    control_iframe.onload = resolve;
    control_iframe.src = ORIGIN + `/common/blank.html`;
    document.body.append(control_iframe);
  });

  const createAnonymousIframe = new Promise(async resolve => {
    anonymous_iframe.onload = resolve;
    anonymous_iframe.src = ORIGIN + `/common/blank.html`;
    anonymous_iframe.anonymous = true;
    document.body.append(anonymous_iframe);
  });

  await Promise.all([createControlIframe, createAnonymousIframe]);
});

// Create cross-origin popup from iframes. The opener should be blocked for
// anonymous iframe and work for normal iframe.
promise_test(async t => {
  const control_token = token();
  const control_src = REMOTE_ORIGIN + executor_path + `&uuid=${control_token}`;
  const control_popup = control_iframe.contentWindow.open(control_src);
  add_completion_callback(() => send(control_token, "close();"));
  assert_equals(
    control_popup.opener, control_iframe.contentWindow,
    "Opener from normal iframe should be available.");

  const anonymous_token = token();
  const anonymous_src =
    REMOTE_ORIGIN + executor_path + `&uuid=${anonymous_token}`;
  const anonymous_popup = anonymous_iframe.contentWindow.open(anonymous_src);
  add_completion_callback(() => send(anonymous_token, "close();"));
  assert_equals(
    anonymous_popup, null, "Opener from anonymous iframe should be blocked.");
}, 'Cross-origin popup from normal/anonymous iframes.');

// Create a same-origin popup from iframes. The opener should be blocked for
// anonymous iframe and work for normal iframe.
promise_test(async t => {
  const control_token = token();
  const control_src = ORIGIN + executor_path + `&uuid=${control_token}`;
  const control_popup = control_iframe.contentWindow.open(control_src);
  add_completion_callback(() => send(control_token, "close();"));
  assert_equals(
    control_popup.opener, control_iframe.contentWindow,
    "Opener from normal iframe should be available.");

  const anonymous_token = token();
  const anonymous_src =
    ORIGIN + executor_path + `&uuid=${anonymous_token}`;
  const anonymous_popup = anonymous_iframe.contentWindow.open(anonymous_src);
  add_completion_callback(() => send(anonymous_token, "close();"));
  assert_equals(
    anonymous_popup, null, "Opener from anonymous iframe should be blocked.");
}, 'Same-origin popup from normal/anonymous iframes.');
