// META: title=Test that promise exception are converted to rejections

var binding = new TestBinding();

/*
  static Promise<any> staticThrowToRejectPromise();
  Promise<any> methodThrowToRejectPromise();
  readonly attribute Promise<any> getterThrowToRejectPromise;

  static Promise<any> staticInternalThrowToRejectPromise([EnforceRange] unsigned long long arg);
  Promise<any> methodInternalThrowToRejectPromise([EnforceRange] unsigned long long arg);
*/

promise_test((t) => {
    return promise_rejects_js(t, TypeError, TestBinding.staticThrowToRejectPromise());
}, "staticThrowToRejectPromise");

promise_test((t) => {
    return promise_rejects_js(t, TypeError, binding.methodThrowToRejectPromise());
}, "methodThrowToRejectPromise");

promise_test((t) => {
    return promise_rejects_js(t, TypeError, binding.getterThrowToRejectPromise);
}, "getterThrowToRejectPromise");

promise_test((t) => {
    return promise_rejects_js(t, TypeError, TestBinding.staticInternalThrowToRejectPromise(Number.MAX_SAFE_INTEGER + 1));
}, "staticInternalThrowToRejectPromise");

promise_test((t) => {
    return promise_rejects_js(t, TypeError, binding.methodInternalThrowToRejectPromise(Number.MAX_SAFE_INTEGER + 1));
}, "methodInternalThrowToRejectPromise");

promise_test((t) => {
    return promise_rejects_js(t, TypeError, new Promise(() => {
        throw new TypeError();
      }));
}, "exception in JS Promise");
