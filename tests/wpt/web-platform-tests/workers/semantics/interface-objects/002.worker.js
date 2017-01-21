importScripts("/resources/testharness.js");
var unexpected = [
  // https://html.spec.whatwg.org/
  "SharedWorkerGlobalScope",
  "AbstractView",
  "AbstractWorker",
  "ApplicationCache",
  "Location",
  "Navigator",
  "Audio",
  "HTMLCanvasElement",
  "Path",
  "TextMetrics",
  "CanvasProxy",
  "CanvasRenderingContext2D",
  "DrawingStyle",
  "CanvasGradient",
  "CanvasPattern",
  "BeforeUnloadEvent",
  "PopStateEvent",
  "HashChangeEvent",
  "PageTransitionEvent",
  // https://dom.spec.whatwg.org/
  "DOMImplementation",
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
for (var i = 0; i < unexpected.length; ++i) {
  test(function () {
    assert_false(unexpected[i] in self);
  }, "The " + unexpected[i] + " interface object should not be exposed.");
}
done();
