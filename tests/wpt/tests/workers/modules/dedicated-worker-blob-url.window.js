function message_from_worker(worker) {
  return new Promise((resolve, reject) => {
    worker.onmessage = e => resolve(e.data);
    worker.onerror = () => reject(new Error('The worker fired an error event.'));
  });
}

function blob_url(code) {
  return URL.createObjectURL(new Blob([code], {type: 'text/javascript'}));
}

promise_test(async () => {
  const worker = new Worker(blob_url('postMessage("OK");'), {type: 'module'});
  assert_equals(await message_from_worker(worker), 'OK');
}, 'Creating a module worker from a blob URL works.');

promise_test(async () => {
  const url = blob_url('postMessage("OK");');
  const worker = new Worker(url, {type: 'module'});
  URL.revokeObjectURL(url);
  assert_equals(await message_from_worker(worker), 'OK');
}, 'Creating a module worker from a blob URL works immediately before revoking.');

promise_test(async () => {
  const imported = blob_url('postMessage("OK");');
  const worker = new Worker(blob_url(`import ${JSON.stringify(imported)};`), {type: 'module'});
  assert_equals(await message_from_worker(worker), 'OK');
}, 'A module worker created from a blob URL can import another blob URL.');

promise_test(async () => {
  const imported = blob_url('postMessage("OK");');
  const url = blob_url(`import ${JSON.stringify(imported)};`);
  const worker = new Worker(url, {type: 'module'});
  URL.revokeObjectURL(url);
  assert_equals(await message_from_worker(worker), 'OK');
}, 'A module worker created from a blob URL revoked immediately after construction can import another blob URL.');
