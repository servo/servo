// META: global=!default,sharedworker

var expected = [
  // https://html.spec.whatwg.org/
  "ApplicationCache",
  "WorkerGlobalScope",
  "SharedWorkerGlobalScope",
  "Worker",
  "SharedWorker",
  "MessagePort",
  "MessageEvent",
  "WorkerNavigator",
  "MessageChannel",
  "WorkerLocation",
  "ImageData",
  "ImageBitmap",
  "CanvasGradient",
  "CanvasPattern",
  "CanvasPath",
  "Path2D",
  "PromiseRejectionEvent",
  "EventSource",
  "WebSocket",
  "CloseEvent",
  "BroadcastChannel",
  // https://tc39.github.io/ecma262/
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
  // https://xhr.spec.whatwg.org/
  "XMLHttpRequestEventTarget",
  "XMLHttpRequestUpload",
  "XMLHttpRequest",
  "ProgressEvent",
  "FormData",
  // https://url.spec.whatwg.org/
  "URL",
  "URLSearchParams",
  // https://w3c.github.io/FileAPI/
  "File",
  "Blob",
  "FileList",
  "FileReader",
  "FileReaderSync",
  // https://dom.spec.whatwg.org/
  "EventTarget",
  "ErrorEvent",
  "Event",
  "CustomEvent",
  // http://heycam.github.io/webidl/
  "DOMException",
  // https://streams.spec.whatwg.org/
  "ReadableStream",
  "WritableStream",
  "ByteLengthQueuingStrategy",
  "CountQueuingStrategy",
  // http://w3c.github.io/IndexedDB/
  "IDBRequest",
  "IDBOpenDBRequest",
  "IDBVersionChangeEvent",
  "IDBFactory",
  "IDBDatabase",
  "IDBObjectStore",
  "IDBIndex",
  "IDBKeyRange",
  "IDBCursor",
  "IDBCursorWithValue",
  "IDBTransaction",
];

for (var i = 0; i < expected.length; ++i) {
  test(function() {
    assert_true(expected[i] in self);
  }, "The " + expected[i] + " interface object should be exposed");
}
