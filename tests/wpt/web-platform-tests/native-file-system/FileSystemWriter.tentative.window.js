// META: script=resources/test-helpers.js
promise_test(async t => cleanupSandboxedFileSystem(),
    'Cleanup to setup test environment');

promise_test(async t => {
    const handle = await createEmptyFile(t, 'empty_blob');
    const writer = await handle.createWriter();

    await writer.write(0, new Blob([]));

    assert_equals(await getFileContents(handle), '');
    assert_equals(await getFileSize(handle), 0);
}, 'write() with an empty blob to an empty file');

promise_test(async t => {
    const handle = await createEmptyFile(t, 'valid_blob');
    const writer = await handle.createWriter();

    await writer.write(0, new Blob(['1234567890']));

    assert_equals(await getFileContents(handle), '1234567890');
    assert_equals(await getFileSize(handle), 10);
}, 'write() a blob to an empty file');

promise_test(async t => {
    const handle = await createEmptyFile(t, 'blob_with_offset');
    const writer = await handle.createWriter();

    await writer.write(0, new Blob(['1234567890']));
    await writer.write(4, new Blob(['abc']));

    assert_equals(await getFileContents(handle), '1234abc890');
    assert_equals(await getFileSize(handle), 10);
}, 'write() called with a blob and a valid offset');

promise_test(async t => {
    const handle = await createEmptyFile(t, 'bad_offset');
    const writer = await handle.createWriter();

    await promise_rejects(t, 'InvalidStateError', writer.write(4, new Blob(['abc'])));

    assert_equals(await getFileContents(handle), '');
    assert_equals(await getFileSize(handle), 0);
}, 'write() called with an invalid offset');

promise_test(async t => {
    const handle = await createEmptyFile(t, 'trunc_shrink');
    const writer = await handle.createWriter();

    await writer.write(0, new Blob(['1234567890']));
    await writer.truncate(5);

    assert_equals(await getFileContents(handle), '12345');
    assert_equals(await getFileSize(handle), 5);
}, 'truncate() to shrink a file');

promise_test(async t => {
    const handle = await createEmptyFile(t, 'trunc_grow');
    const writer = await handle.createWriter();

    await writer.write(0, new Blob(['abc']));
    await writer.truncate(5);

    assert_equals(await getFileContents(handle), 'abc\0\0');
    assert_equals(await getFileSize(handle), 5);
}, 'truncate() to grow a file');

promise_test(async t => {
    const handle = await createEmptyFile(t, 'write_stream');
    const writer = await handle.createWriter();

    const stream = new Response('1234567890').body;
    await writer.write(0, stream);

    assert_equals(await getFileContents(handle), '1234567890');
    assert_equals(await getFileSize(handle), 10);
}, 'write() called with a ReadableStream');

promise_test(async t => {
    const handle = await createEmptyFile(t, 'write_stream');
    const handle_writer = await handle.createWriter();

    const { writable, readable } = new TransformStream();
    const write_result = handle_writer.write(0, readable);

    const stream_writer = writable.getWriter();
    stream_writer.write(new Uint8Array([0x73, 0x74, 0x72, 0x65, 0x61, 0x6D, 0x73, 0x21]));
    garbageCollect();
    stream_writer.write(new Uint8Array([0x21, 0x21]));
    stream_writer.close();

    await write_result;

    assert_equals(await getFileContents(handle), 'streams!!!');
    assert_equals(await getFileSize(handle), 10);
}, 'Using a WritableStream writer to write');
