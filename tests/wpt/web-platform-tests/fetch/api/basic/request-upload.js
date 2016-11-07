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

done();
