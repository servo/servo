'use strict';

self.getterRejects = (t, obj, getterName, target) => {
  const getter = Object.getOwnPropertyDescriptor(obj, getterName).get;

  return promise_rejects(t, new TypeError(), getter.call(target));
};

self.methodRejects = (t, obj, methodName, target) => {
  const method = obj[methodName];

  return promise_rejects(t, new TypeError(), method.call(target));
};

self.getterThrows = (obj, getterName, target) => {
  const getter = Object.getOwnPropertyDescriptor(obj, getterName).get;

  assert_throws(new TypeError(), () => getter.call(target), getterName + ' should throw a TypeError');
};

self.methodThrows = (obj, methodName, target, args) => {
  const method = obj[methodName];

  assert_throws(new TypeError(), () => method.apply(target, args), methodName + ' should throw a TypeError');
};

self.garbageCollect = () => {
  if (self.gc) {
    // Use --expose_gc for V8 (and Node.js)
    // Exposed in SpiderMonkey shell as well
    self.gc();
  } else if (self.GCController) {
    // Present in some WebKit development environments
    GCController.collect();
  } else {
    /* eslint-disable no-console */
    console.warn('Tests are running without the ability to do manual garbage collection. They will still work, but ' +
      'coverage will be suboptimal.');
    /* eslint-enable no-console */
  }
};

self.delay = ms => new Promise(resolve => step_timeout(resolve, ms));
