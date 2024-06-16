// META: global=window,worker
// META: script=resources/test-helpers.js

promise_test(async t => {
    const directory = await navigator.storage.getDirectory();
    return directory.getFileHandle("testFile", { create: true });
}, "Call getFileHandle successfully");

promise_test(async t => {
    const directory = await navigator.storage.getDirectory();
    return directory.getDirectoryHandle("testDirectory", { create: true });
}, "Call getDirectoryHandle successfully");
