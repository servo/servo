// META: global=window,worker
// META: script=../resources/utils.js
// META: script=/common/utils.js
// META: script=/common/get-host-info.sub.js

function testUpload(desc, url, method, createBody, expectedBody) {
  const requestInit = {method};
  promise_test(async function(){
    const body = createBody();
    if (body) {
      requestInit["body"] = body;
    }
    const resp = await fetch(url, requestInit);
    const text = await resp.text();
    assert_equals(text, expectedBody);
  }, desc);
}

const url = RESOURCES_DIR + "echo-content.h2.py"

testUpload("Fetch with POST with empty ReadableStream", url,
  "POST",
  () => {
    return new ReadableStream({start: controller => {
      controller.close();
    }})
  },
  "");

testUpload("Fetch with POST with ReadableStream", url,
  "POST",
  () => {
    return new ReadableStream({start: controller => {
      const encoder = new TextEncoder();
      controller.enqueue(encoder.encode("Test"));
      controller.close();
    }})
  },
  "Test");
