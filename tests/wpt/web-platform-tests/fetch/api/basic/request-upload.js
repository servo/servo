if (this.document === undefined) {
  importScripts("/resources/testharness.js");
  importScripts("../resources/utils.js");
}

function testUpload(desc, url, method, body, expectedBody) {
  var requestInit = {"method": method}
  promise_test(function(test){
    if (typeof body === "function")
      body = body();
    if (body)
      requestInit["body"] = body;
    return fetch(url, requestInit).then(function(resp) {
      return resp.text().then((text)=> {
        assert_equals(text, expectedBody);
      });
    });
  }, desc);
}

function testUploadFailure(desc, url, method, body) {
  const requestInit = {"method": method};
  promise_test(test => {
    if (typeof body === "function")
      body = body();
    if (body)
      requestInit["body"] = body;
    return promise_rejects(new TypeError(), fetch(url, requestInit));
  }, desc);
}

var url = RESOURCES_DIR + "echo-content.py"

testUpload("Fetch with PUT with body", url, "PUT", "Request's body", "Request's body");
testUpload("Fetch with POST with text body", url, "POST", "Request's body", "Request's body");
testUpload("Fetch with POST with URLSearchParams body", url, "POST", function() { return new URLSearchParams("name=value"); }, "name=value");
testUpload("Fetch with POST with Blob body", url, "POST", new Blob(["Test"]), "Test");
testUpload("Fetch with POST with ArrayBuffer body", url, "POST", new ArrayBuffer(4), "\0\0\0\0");
testUpload("Fetch with POST with Uint8Array body", url, "POST", new Uint8Array(4), "\0\0\0\0");
testUpload("Fetch with POST with Int8Array body", url, "POST", new Int8Array(4), "\0\0\0\0");
testUpload("Fetch with POST with Float32Array body", url, "POST", new Float32Array(1), "\0\0\0\0");
testUpload("Fetch with POST with Float64Array body", url, "POST", new Float64Array(1), "\0\0\0\0\0\0\0\0");
testUpload("Fetch with POST with DataView body", url, "POST", new DataView(new ArrayBuffer(8), 0, 4), "\0\0\0\0");
testUpload("Fetch with POST with Blob body with mime type", url, "POST", new Blob(["Test"], { type: "text/maybe" }), "Test");
testUpload("Fetch with POST with ReadableStream", url, "POST", new ReadableStream({start: controller => {
    const encoder = new TextEncoder();
    controller.enqueue(encoder.encode("Test"));
    controller.close();
  }}), "Test");
testUploadFailure("Fetch with POST with ReadableStream containing String", url, "POST", new ReadableStream({start: controller => {
    controller.enqueue("Test");
    controller.close();
  }}));
testUploadFailure("Fetch with POST with ReadableStream containing null", url, "POST", new ReadableStream({start: controller => {
    controller.enqueue(null);
    controller.close();
  }}));
testUploadFailure("Fetch with POST with ReadableStream containing number", url, "POST", new ReadableStream({start: controller => {
    controller.enqueue(99);
    controller.close();
  }}));
testUploadFailure("Fetch with POST with ReadableStream containing ArrayBuffer", url, "POST", new ReadableStream({start: controller => {
    controller.enqueue(new ArrayBuffer());
    controller.close();
  }}));
testUploadFailure("Fetch with POST with ReadableStream containing Blob", url, "POST", new ReadableStream({start: controller => {
    controller.enqueue(new Blob());
    controller.close();
  }}));

done();
