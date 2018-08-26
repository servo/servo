// META: global=window,dedicatedworker,sharedworker

function assert_xhr(stream) {
  const client = new XMLHttpRequest();
  client.open("POST", "...");
  assert_throws(new TypeError(), () => client.send(stream));
}

test(() => {
  const stream = new ReadableStream();
  stream.getReader();
  assert_xhr(stream);
}, "XMLHttpRequest: send() with a stream on which getReader() is called");

test(() => {
  const stream = new ReadableStream();
  stream.getReader().read();
  assert_xhr(stream);
}, "XMLHttpRequest: send() with a stream on which read() is called");

promise_test(async () => {
  const stream = new ReadableStream({ pull: c => c.enqueue(new Uint8Array()) }),
        reader = stream.getReader();
  await reader.read();
  reader.releaseLock();
  assert_xhr(stream);
}, "XMLHttpRequest: send() with a stream on which read() and releaseLock() are called");
