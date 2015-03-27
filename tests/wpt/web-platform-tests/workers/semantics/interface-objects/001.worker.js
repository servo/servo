importScripts("/resources/testharness.js");
var expected = [
  "WorkerGlobalScope",
  "EventTarget",
  "DedicatedWorkerGlobalScope",
  "ErrorEvent",
  "Event",
  "Worker",
  "DOMException",
  "SharedWorker",
  "MessagePort",
  "MessageEvent",
  "WorkerNavigator",
  "MessageChannel",
  "WorkerLocation",
  "ImageData",
  "File",
  "Blob",
  "FileList",
  "XMLHttpRequest",
  "ProgressEvent",
  "FormData",
  "ArrayBuffer",
  "Int8Array",
  "Uint8Array",
  "Uint8ClampedArray",
  "Int16Array",
  "Uint16Array",
  "Int32Array",
  "Uint32Array",
  "Float32Array",
  "Float64Array",
  "DataView",
  "CanvasProxy",
  "ImageBitmap",
  "CanvasRenderingContext2D",
  "DrawingStyle",
  "CanvasGradient",
  "CanvasPattern",
  "Path",
  "TextMetrics"
];
for (var i = 0; i < expected.length; ++i) {
  test(function () {
    assert_own_property(self, expected[i]);
  }, "The " + expected[i] + " interface object should be exposed.");
}
done();
