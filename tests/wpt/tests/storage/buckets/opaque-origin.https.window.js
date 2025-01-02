// META: title=Storage Buckets API: Interface is not exposed in opaque origins.
// META: script=resources/util.js
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

// |kSandboxWindowUrl| sends the result of methods on StorageBucketManager.
// For windows using sandbox="allow-scripts", it must produce a rejected
// promise.
async function verify_results_from_sandboxed_child_window(test) {
  const event_watcher = new EventWatcher(test, self, 'message');

  const first_message_event = await event_watcher.wait_for('message');
  assert_equals(
    first_message_event.data,
    'navigator.storageBuckets.open(): REJECTED: SecurityError');

  const second_message_event = await event_watcher.wait_for('message');
  assert_equals(
    second_message_event.data,
    'navigator.storageBuckets.keys(): REJECTED: SecurityError');

  const third_message_event = await event_watcher.wait_for('message');
  assert_equals(
    third_message_event.data,
    'navigator.storageBuckets.delete(): REJECTED: SecurityError');
}

promise_test(async testCase => {
  prepareForBucketTest(testCase);
  add_iframe(testCase, kSandboxWindowUrl, /*sandbox=*/ 'allow-scripts');
  await verify_results_from_sandboxed_child_window(testCase);
}, 'StorageBucketManager methods must reject in a sandboxed iframe.');

promise_test(async testCase => {
  prepareForBucketTest(testCase);
  const child_window_url = kSandboxWindowUrl +
      '?pipe=header(Content-Security-Policy, sandbox allow-scripts)';

  const child_window = window.open(child_window_url);
  testCase.add_cleanup(() => {
    child_window.close();
  });

  await verify_results_from_sandboxed_child_window(testCase);
}, 'StorageBucketManager methods must reject in a sandboxed opened window.');
