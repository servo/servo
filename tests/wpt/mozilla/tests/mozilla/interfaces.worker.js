importScripts("/resources/testharness.js");
importScripts("interfaces.js");

// This is a list of interfaces that are exposed to every web worker.
// Please only add things to this list with great care and proper review
// from the associated module peers.

// IMPORTANT: Do not change the list below without review from a DOM peer!
test_interfaces([
  "Blob",
  "CloseEvent",
  "DOMMatrix",
  "DOMMatrixReadOnly",
  "DOMPoint",
  "DOMPointReadOnly",
  "DOMQuad",
  "DOMRect",
  "DOMRectReadOnly",
  "CustomEvent",
  "DedicatedWorkerGlobalScope",
  "DOMException",
  "ErrorEvent",
  "Event",
  "EventSource",
  "EventTarget",
  "File",
  "FileList",
  "FileReader",
  "FileReaderSync",
  "FormData",
  "Headers",
  "History",
  "ImageData",
  "MessageEvent",
  "Performance",
  "PerformanceEntry",
  "PerformanceMark",
  "PerformanceMeasure",
  "PerformanceObserver",
  "PerformanceObserverEntryList",
  "PerformancePaintTiming",
  "ProgressEvent",
  "PromiseRejectionEvent",
  "Request",
  "Response",
  "TextDecoder",
  "TextEncoder",
  "URL",
  "URLSearchParams",
  "WebSocket",
  "Worker",
  "WorkerGlobalScope",
  "WorkerLocation",
  "WorkerNavigator",
  "XMLHttpRequest",
  "XMLHttpRequestEventTarget",
  "XMLHttpRequestUpload",
  "console",
]);

done();
