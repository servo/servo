// META: script=resources/test-helpers.js
// META: script=resources/sandboxed-fs-test-helpers.js
// META: script=script-tests/FileSystemBaseHandle-remove.js

directory_test(async (t, root) => {
    await promise_rejects_dom(t, 'InvalidStateError', root.remove());
}, 'cannot remove the root of a sandbox file system');
