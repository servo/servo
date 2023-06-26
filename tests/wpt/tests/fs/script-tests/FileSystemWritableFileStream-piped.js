'use strict';

directory_test(async (t, root) => {
  const handle = await createEmptyFile(t, 'foo_string.txt', root);
  const wfs = await handle.createWritable();

  const rs = recordingReadableStream({
    start(controller) {
      controller.enqueue('foo_string');
      controller.close();
    }
  });

  await rs.pipeTo(wfs, { preventCancel: true });
  assert_equals(await getFileContents(handle), 'foo_string');
  assert_equals(await getFileSize(handle), 10);
}, 'can be piped to with a string');

directory_test(async (t, root) => {
  const handle = await createEmptyFile(t, 'foo_arraybuf.txt', root);
  const wfs = await handle.createWritable();
  const buf = new ArrayBuffer(3);
  const intView = new Uint8Array(buf);
  intView[0] = 0x66;
  intView[1] = 0x6f;
  intView[2] = 0x6f;

  const rs = recordingReadableStream({
    start(controller) {
      controller.enqueue(buf);
      controller.close();
    }
  });

  await rs.pipeTo(wfs, { preventCancel: true });
  assert_equals(await getFileContents(handle), 'foo');
  assert_equals(await getFileSize(handle), 3);
}, 'can be piped to with an ArrayBuffer');

directory_test(async (t, root) => {
  const handle = await createEmptyFile(t, 'foo_blob.txt', root);
  const wfs = await handle.createWritable();

  const rs = recordingReadableStream({
    start(controller) {
      controller.enqueue(new Blob(['foo']));
      controller.close();
    }
  });

  await rs.pipeTo(wfs, { preventCancel: true });
  assert_equals(await getFileContents(handle), 'foo');
  assert_equals(await getFileSize(handle), 3);
}, 'can be piped to with a Blob');

directory_test(async (t, root) => {
  const handle = await createEmptyFile(t, 'foo_write_param.txt', root);
  const wfs = await handle.createWritable();

  const rs = recordingReadableStream({
    start(controller) {
      controller.enqueue({type: 'write', data: 'foobar'});
      controller.close();
    }
  });

  await rs.pipeTo(wfs, { preventCancel: true });
  assert_equals(await getFileContents(handle), 'foobar');
  assert_equals(await getFileSize(handle), 6);
}, 'can be piped to with a param object with write command');

directory_test(async (t, root) => {
  const handle = await createEmptyFile(t, 'foo_write_param.txt', root);
  const wfs = await handle.createWritable();

  const rs = recordingReadableStream({
    start(controller) {
      controller.enqueue({type: 'write', data: 'foobar'});
      controller.enqueue({type: 'truncate', size: 10});
      controller.enqueue({type: 'write', position: 0, data: 'baz'});
      controller.close();
    }
  });

  await rs.pipeTo(wfs, { preventCancel: true });
  assert_equals(await getFileContents(handle), 'bazbar\0\0\0\0');
  assert_equals(await getFileSize(handle), 10);
}, 'can be piped to with a param object with multiple commands');

directory_test(async (t, root) => {
  const handle = await createEmptyFile(t, 'foo_write_queued.txt', root);
  const wfs = await handle.createWritable();

  const rs = recordingReadableStream({
    start(controller) {
      controller.enqueue('foo');
      controller.enqueue('bar');
      controller.enqueue('baz');
      controller.close();
    }
  });

  await rs.pipeTo(wfs, { preventCancel: true });
  assert_equals(await getFileContents(handle), 'foobarbaz');
  assert_equals(await getFileSize(handle), 9);
}, 'multiple operations can be queued');

directory_test(async (t, root) => {
  const handle = await createEmptyFile(t, 'fetched.txt', root);
  const wfs = await handle.createWritable();

  const response = await fetch('data:text/plain,fetched from far');
  const body = await response.body;
  await body.pipeTo(wfs, { preventCancel: true });
  assert_equals(await getFileContents(handle), 'fetched from far');
  assert_equals(await getFileSize(handle), 16);
}, 'plays well with fetch');

directory_test(async (t, root) => {
  const handle = await createEmptyFile(t, 'aborted should_be_empty.txt', root);
  const wfs = await handle.createWritable();

  const response = await fetch('data:text/plain,fetched from far');
  const body = await response.body;

  const abortController = new AbortController();
  const signal = abortController.signal;

  const promise = body.pipeTo(wfs, { signal });
  await abortController.abort();

  await promise_rejects_dom(t, 'AbortError', promise, 'stream is aborted');
  await promise_rejects_js(t, TypeError, wfs.close(), 'stream cannot be closed to flush writes');

  assert_equals(await getFileContents(handle), '');
  assert_equals(await getFileSize(handle), 0);
}, 'abort() aborts write');
