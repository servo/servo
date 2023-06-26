// META: timeout=long
// META: script=/common/get-host-info.sub.js
// META: script=/common/utils.js
// META: script=/common/dispatcher/dispatcher.js
// META: script=/html/cross-origin-embedder-policy/credentialless/resources/common.js

const {ORIGIN, REMOTE_ORIGIN} = get_host_info();
const control_iframe = document.createElement('iframe');
const iframe_credentialless = document.createElement('iframe');

promise_setup(async t => {
  const createControlIframe = new Promise(async resolve => {
    control_iframe.onload = resolve;
    control_iframe.src = ORIGIN + `/common/blank.html`;
    document.body.append(control_iframe);
  });

  const createIframeCredentialless = new Promise(async resolve => {
    iframe_credentialless.onload = resolve;
    iframe_credentialless.src = ORIGIN + `/common/blank.html`;
    iframe_credentialless.credentialless = true;
    document.body.append(iframe_credentialless);
  });

  await Promise.all([createControlIframe, createIframeCredentialless]);
});

// Create cross-origin popup from iframes. The opener should be blocked for
// credentialless iframe and work for normal iframe.
promise_test(async t => {
  const control_token = token();
  const control_src = REMOTE_ORIGIN + executor_path + `&uuid=${control_token}`;
  const control_popup = control_iframe.contentWindow.open(control_src);
  add_completion_callback(() => send(control_token, "close();"));
  assert_equals(
    control_popup.opener, control_iframe.contentWindow,
    "Opener from normal iframe should be available.");

  const credentialless_token = token();
  const credentialless_src =
    REMOTE_ORIGIN + executor_path + `&uuid=${credentialless_token}`;
  const credentialless_popup =
    iframe_credentialless.contentWindow.open(credentialless_src);
  add_completion_callback(() => send(credentialless_token, "close();"));
  assert_equals(credentialless_popup, null,
    "Opener from credentialless iframe should be blocked.");
}, 'Cross-origin popup from normal/credentiallessiframes.');

// Create a same-origin popup from iframes. The opener should be blocked for
// credentialless iframe and work for normal iframe.
promise_test(async t => {
  const control_token = token();
  const control_src = ORIGIN + executor_path + `&uuid=${control_token}`;
  const control_popup = control_iframe.contentWindow.open(control_src);
  add_completion_callback(() => send(control_token, "close();"));
  assert_equals(
    control_popup.opener, control_iframe.contentWindow,
    "Opener from normal iframe should be available.");

  const credentialless_token = token();
  const credentialless_src =
    ORIGIN + executor_path + `&uuid=${credentialless_token}`;
  const credentialless_popup = iframe_credentialless.contentWindow.open(credentialless_src);
  add_completion_callback(() => send(credentialless_token, "close();"));
  assert_equals(credentialless_popup, null,
    "Opener from credentialless iframe should be blocked.");
}, 'Same-origin popup from normal/credentialless iframes.');
