// META: global=serviceworker

promise_test(async t => {
  assert_throws_js(ReferenceError, () => new FileSystemObserver(() => {}));
}, 'Creating a FileSystemObserver from an unsupported global fails');
