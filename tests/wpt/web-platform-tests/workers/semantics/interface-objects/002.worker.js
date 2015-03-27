importScripts("/resources/testharness.js");
var unexpected = [
  "AbstractView",
  "AbstractWorker",
  "ApplicationCache",
  "Location",
  "Navigator",
  "DOMImplementation",
  "Audio",
  "HTMLCanvasElement",
  "MouseEvent",
];
for (var i = 0; i < unexpected.length; ++i) {
  test(function () {
    assert_false(unexpected[i] in self);
  }, "The " + unexpected[i] + " interface object should not be exposed.");
}
done();
