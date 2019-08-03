onconnect = function(e) {
  var unexpected = [
    // https://html.spec.whatwg.org/
    "DedicatedWorkerGlobalScope",
    "AbstractView",
    "AbstractWorker",
    "Location",
    "Navigator",
    "DOMImplementation",
    "Audio",
    "HTMLCanvasElement",
    "Path",
    "TextMetrics",
    "CanvasProxy",
    "CanvasRenderingContext2D",
    "DrawingStyle",
    "PopStateEvent",
    "HashChangeEvent",
    "PageTransitionEvent",
    // https://streams.spec.whatwg.org/
    "ReadableStreamDefaultReader",
    "ReadableStreamBYOBReader",
    "ReadableStreamDefaultController",
    "ReadableByteStreamController",
    "WritableStreamDefaultWriter",
    "WritableStreamDefaultController",
    // http://w3c.github.io/IndexedDB/
    "IDBEnvironment",
    // https://www.w3.org/TR/2010/NOTE-webdatabase-20101118/
    "Database",
    // https://w3c.github.io/uievents/
    "UIEvent",
    "FocusEvent",
    "MouseEvent",
    "WheelEvent",
    "InputEvent",
    "KeyboardEvent",
    "CompositionEvent",
  ];
  var result = [];
  for (var i = 0; i < unexpected.length; ++i) {
    result.push([unexpected[i], unexpected[i] in self]);
  }
  e.ports[0].postMessage(result);
}