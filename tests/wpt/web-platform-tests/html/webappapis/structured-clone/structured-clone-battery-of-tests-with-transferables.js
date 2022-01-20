structuredCloneBatteryOfTests.push({
  description: 'ArrayBuffer',
  async f(runner) {
    const buffer = new Uint8Array([1]).buffer;
    const copy = await runner.structuredClone(buffer, [buffer]);
    assert_equals(buffer.byteLength, 0);
    assert_equals(copy.byteLength, 1);
  }
});

structuredCloneBatteryOfTests.push({
  description: 'MessagePort',
  async f(runner) {
    const {port1, port2} = new MessageChannel();
    const copy = await runner.structuredClone(port2, [port2]);
    const msg = new Promise(resolve => port1.onmessage = resolve);
    copy.postMessage('ohai');
    assert_equals((await msg).data, 'ohai');
  }
});

// TODO: ImageBitmap

structuredCloneBatteryOfTests.push({
  description: 'A detached ArrayBuffer cannot be transferred',
  async f(runner, t) {
    const buffer = new ArrayBuffer();
    await runner.structuredClone(buffer, [buffer]);
    await promise_rejects_dom(
      t,
      "DataCloneError",
      runner.structuredClone(buffer, [buffer])
    );
  }
});

structuredCloneBatteryOfTests.push({
  description: 'A detached platform object cannot be transferred',
  async f(runner, t) {
    const {port1} = new MessageChannel();
    await runner.structuredClone(port1, [port1]);
    await promise_rejects_dom(
      t,
      "DataCloneError",
      runner.structuredClone(port1, [port1])
    );
  }
});

structuredCloneBatteryOfTests.push({
  description: 'Transferring a non-transferable platform object fails',
  async f(runner, t) {
    const blob = new Blob();
    await promise_rejects_dom(
      t,
      "DataCloneError",
      runner.structuredClone(blob, [blob])
    );
  }
});

structuredCloneBatteryOfTests.push({
  description: 'An object whose interface is deleted from the global object must still be received',
  async f(runner) {
    const {port1} = new MessageChannel();
    const messagePortInterface = globalThis.MessagePort;
    delete globalThis.MessagePort;
    try {
      const transfer = await runner.structuredClone(port1, [port1]);
      assert_true(transfer instanceof messagePortInterface);
    } finally {
      globalThis.MessagePort = messagePortInterface;
    }
  }
});

structuredCloneBatteryOfTests.push({
  description: 'A subclass instance will be received as its closest transferable superclass',
  async f(runner) {
    // MessagePort doesn't have a constructor, so we must use something else.

    // Make sure that ReadableStream is transferable before we test its subclasses.
    try {
      const stream = new ReadableStream();
      await runner.structuredClone(stream, [stream]);
    } catch(err) {
      if (err instanceof DOMException && err.code === DOMException.DATA_CLONE_ERR) {
        throw new OptionalFeatureUnsupportedError("ReadableStream isn't transferable");
      } else {
        throw err;
      }
    }

    class ReadableStreamSubclass extends ReadableStream {}
    const original = new ReadableStreamSubclass();
    const transfer = await runner.structuredClone(original, [original]);
    assert_equals(Object.getPrototypeOf(transfer), ReadableStream.prototype);
  }
});
