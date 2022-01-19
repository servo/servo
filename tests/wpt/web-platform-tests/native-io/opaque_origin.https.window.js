// META: title=NativeIO API: Interface is not exposed in opaque origins.
// META: global=window

const kSandboxWindowUrl = 'resources/opaque-origin-sandbox.html';

function add_iframe(test, src, sandbox) {
  const iframe = document.createElement('iframe');
  iframe.src = src;
  if (sandbox !== undefined) {
    iframe.sandbox = sandbox;
  }
  document.body.appendChild(iframe);
  test.add_cleanup(() => {
    iframe.remove();
  });
}

// Creates a data URI iframe that uses postMessage() to provide its parent
// with the test result. The iframe checks for the existence of
// |interface_name| on the window.
async function verify_does_not_exist_in_data_uri_iframe(
  test, interface_name) {
  const iframe_content =
    '<script>' +
    '  const is_interface_defined = ' +
    `    (typeof ${interface_name} !== 'undefined');` +
    '  parent.postMessage({is_interface_defined}, "*")' +
    '</script>';

  const data_uri = `data:text/html,${encodeURIComponent(iframe_content)}`;
  add_iframe(test, data_uri);

  const event_watcher = new EventWatcher(test, self, 'message');
  const message_event = await event_watcher.wait_for('message')

  assert_false(message_event.data.is_interface_defined,
    `Data URI iframes must not define '${interface_name}'.`);
}

// |kSandboxWindowUrl| informs this window if storageFoundation is non-null.
async function verify_results_from_sandboxed_child_window(test) {
  const event_watcher = new EventWatcher(test, self, 'message');

  const message = await event_watcher.wait_for('message');
  assert_false(
    message.data, 'Sandboxed iframes must not define storageFoundation.');
}

promise_test(async testCase => {
  await verify_does_not_exist_in_data_uri_iframe(testCase, 'storageFoundation');
}, 'storageFoundation must be undefined for data URI iframes.');

promise_test(async testCase => {
  add_iframe(testCase, kSandboxWindowUrl, /*sandbox=*/ 'allow-scripts');
  await verify_results_from_sandboxed_child_window(testCase);
}, 'storageFoundation must be null in a sandboxed iframe.');


promise_test(
  async testCase => {
    const child_window_url = kSandboxWindowUrl +
        '?pipe=header(Content-Security-Policy, sandbox allow-scripts)';

    const child_window = window.open(child_window_url);
    testCase.add_cleanup(() => {
      child_window.close();
    });

    await verify_results_from_sandboxed_child_window(testCase);
  },
  'storageFoundation must be null in a sandboxed opened window.');
