// META: global=window,worker
// META: script=/common/gc.js

promise_test(async () => {
  let i = 0;
  const repeat = 5;
  const buffer = await new Response(new ReadableStream({
    pull(c) {
      if (i >= repeat) {
        c.close();
        return;
      }
      ++i;
      c.enqueue(new Uint8Array([0]))
      garbageCollect();
    }
  })).arrayBuffer();
  assert_equals(buffer.byteLength, repeat, `The buffer should be ${repeat}-byte long`);
}, "GC/CC should not abruptly close the stream while being consumed by Response");

// A body extracted from a FormData object keeps that object as its source, which keeps its File
// entries, and so their blob URL registrations, alive for as long as the body can be read.
function blobFormData() {
  const formData = new FormData();
  formData.append("a", new Blob(["contents"], { type: "text/plain" }));
  return formData;
}

promise_test(async () => {
  const response = new Response(blobFormData());
  await garbageCollect();
  const formData = await response.formData();
  assert_equals(await formData.get("a").text(), "contents");
}, "GC/CC should not discard a Response's FormData body Blob before it is read");

promise_test(async () => {
  const request = new Request("/", { method: "POST", body: blobFormData() });
  await garbageCollect();
  const formData = await request.formData();
  assert_equals(await formData.get("a").text(), "contents");
}, "GC/CC should not discard a Request's FormData body Blob before it is read");

promise_test(async () => {
  const clone = new Response(blobFormData()).clone();
  await garbageCollect();
  const formData = await clone.formData();
  assert_equals(await formData.get("a").text(), "contents");
}, "GC/CC should not discard a cloned FormData body's Blob before it is read");

promise_test(async () => {
  // Constructing a Request from a Request moves the body over, source included.
  const request = new Request(new Request("/", { method: "POST", body: blobFormData() }));
  await garbageCollect();
  const formData = await request.formData();
  assert_equals(await formData.get("a").text(), "contents");
}, "GC/CC should not discard a moved FormData body's Blob before it is read");

promise_test(async () => {
  const response = new Response(blobFormData());
  await garbageCollect();
  const text = await new Response(response.body).text();
  assert_true(text.includes("contents"), "the part's contents should survive");
}, "GC/CC should not discard a FormData body's Blob before its stream is read");
