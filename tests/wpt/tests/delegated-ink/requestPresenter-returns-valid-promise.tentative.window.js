test(() => {
  assert_implements(typeof navigator.ink !== "undefined", 'ink is not supported');
}, "navigator needs to support ink to run this test.");

promise_test(t => {
  return promise_rejects_js(t, TypeError, navigator.ink.requestPresenter('invalid-param'));
}, "Receive rejected promise for an invalid param.");

promise_test(t => {
  var canvas = document.createElement("canvas");
  var fragment = document.createElement("template").content;
  fragment.appendChild(canvas);
  return promise_rejects_dom(t, 'NotAllowedError',
      navigator.ink.requestPresenter({presentationArea: canvas}));
}, "Received rejected promise for presentation area not in the document.");

promise_test(() => {
  return navigator.ink.requestPresenter();
}, "Received fulfilled promise for no param");

promise_test(() => {
  return navigator.ink.requestPresenter(null);
}, "Received fulfilled promise for null param");

promise_test(() => {
  return navigator.ink.requestPresenter({});
}, "Received fulfilled promise for empty dictionary param");

promise_test(() => {
  return navigator.ink.requestPresenter({presentationArea: null});
}, "Received fulfilled promise for dictionary param with valid element.");
