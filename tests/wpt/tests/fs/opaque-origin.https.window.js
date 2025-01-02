'use strict';

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
// |property_name| on the window.
async function verify_does_exist_in_data_uri_iframe(
  test, property_name) {
  const iframe_content =
    '<script>' +
    '  const is_property_name_defined = ' +
    `    (self.${property_name} !== undefined);` +
    '  parent.postMessage({is_property_name_defined}, "*")' +
    '</script>';

  const data_uri = `data:text/html,${encodeURIComponent(iframe_content)}`;
  add_iframe(test, data_uri);

  const event_watcher = new EventWatcher(test, self, 'message');
  const message_event = await event_watcher.wait_for('message')

  assert_true(message_event.data.is_property_name_defined,
    `Data URI iframes must define '${property_name}'.`);
}

// |kSandboxWindowUrl| sends the result of navigator.storage.getDirectory() to
// this window. For windows using sandbox='allow-scripts', this must produce a
// rejected promise.
async function verify_results_from_sandboxed_child_window(test) {
  const event_watcher = new EventWatcher(test, self, 'message');

  const message_event = await event_watcher.wait_for('message');
  assert_equals(message_event.data,
    'navigator.storage.getDirectory(): REJECTED: SecurityError');
}

promise_test(async test => {
  await verify_does_exist_in_data_uri_iframe(
    test, 'FileSystemDirectoryHandle');
}, 'FileSystemDirectoryHandle must be defined for data URI iframes.');

promise_test(
    async test => {
      add_iframe(test, kSandboxWindowUrl, /*sandbox=*/ 'allow-scripts');
      await verify_results_from_sandboxed_child_window(test);
    },
    'navigator.storage.getDirectory() must reject in a sandboxed iframe.');

promise_test(
    async test => {
      const child_window_url = kSandboxWindowUrl +
          '?pipe=header(Content-Security-Policy, sandbox allow-scripts)';

      const child_window = window.open(child_window_url);
      test.add_cleanup(() => {
        child_window.close();
      });

      await verify_results_from_sandboxed_child_window(test);
    },
    'navigator.storage.getDirectory() ' +
        'must reject in a sandboxed opened window.');
