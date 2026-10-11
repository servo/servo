// META: title=FileSystemHandle cannot be posted to a non-secure context of the same origin
// META: script=/common/get-host-info.sub.js
'use strict';

const kTargetURL = `${location.origin}/fs/resources/insecure-context-message-target.html`;
const kHostURL = `${get_host_info().UNAUTHENTICATED_ORIGIN}/fs/resources/insecure-context-message-host.html?target=${encodeURIComponent(kTargetURL)}`;

function nextMessage() {
  return new Promise(resolve => window.addEventListener('message', e => resolve(e.data), { once: true }));
}

let target;
promise_setup(async () => {
  const ready = nextMessage();
  const host = window.open(kHostURL);
  add_completion_callback(() => host.close());
  const data = await ready;
  assert_equals(data.type, 'ready');
  assert_false(data.isSecureContext, 'the target is not a secure context');
  target = host.frames[0];
});

async function post(value) {
  const result = nextMessage();
  target.postMessage(value, location.origin);
  return (await result).type;
}

promise_test(async () => {
  assert_equals(await post({ value: 1 }), 'message');
}, 'A plain object can be posted to a non-secure context of the same origin');

promise_test(async t => {
  const root = await navigator.storage.getDirectory();
  const file = await root.getFileHandle('insecure-context-file', { create: true });
  t.add_cleanup(() => root.removeEntry(file.name));
  assert_equals(await post(file), 'messageerror');
}, 'FileSystemFileHandle cannot be posted to a non-secure context of the same origin');

promise_test(async t => {
  const root = await navigator.storage.getDirectory();
  const directory = await root.getDirectoryHandle('insecure-context-directory', { create: true });
  t.add_cleanup(() => root.removeEntry(directory.name));
  assert_equals(await post(directory), 'messageerror');
}, 'FileSystemDirectoryHandle cannot be posted to a non-secure context of the same origin');
