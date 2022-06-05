'use strict';

self.getterRejects = (t, obj, getterName, target) => {
  const getter = Object.getOwnPropertyDescriptor(obj, getterName).get;

  return promise_rejects_js(t, TypeError, getter.call(target), getterName + ' should reject with a TypeError');
};

self.getterRejectsForAll = (t, obj, getterName, targets) => {
  return Promise.all(targets.map(target => self.getterRejects(t, obj, getterName, target)));
};

self.methodRejects = (t, obj, methodName, target, args) => {
  const method = obj[methodName];

  return promise_rejects_js(t, TypeError, method.apply(target, args),
                         methodName + ' should reject with a TypeError');
};

self.methodRejectsForAll = (t, obj, methodName, targets, args) => {
  return Promise.all(targets.map(target => self.methodRejects(t, obj, methodName, target, args)));
};

self.getterThrows = (obj, getterName, target) => {
  const getter = Object.getOwnPropertyDescriptor(obj, getterName).get;

  assert_throws_js(TypeError, () => getter.call(target), getterName + ' should throw a TypeError');
};

self.getterThrowsForAll = (obj, getterName, targets) => {
  targets.forEach(target => self.getterThrows(obj, getterName, target));
};

self.methodThrows = (obj, methodName, target, args) => {
  const method = obj[methodName];
  assert_equals(typeof method, 'function', methodName + ' should exist');

  assert_throws_js(TypeError, () => method.apply(target, args), methodName + ' should throw a TypeError');
};

self.methodThrowsForAll = (obj, methodName, targets, args) => {
  targets.forEach(target => self.methodThrows(obj, methodName, target, args));
};

self.constructorThrowsForAll = (constructor, firstArgs) => {
  firstArgs.forEach(firstArg => assert_throws_js(TypeError, () => new constructor(firstArg),
                                                 'constructor should throw a TypeError'));
};

self.garbageCollect = async () => {
  if (self.TestUtils?.gc) {
    // https://testutils.spec.whatwg.org/#the-testutils-namespace
    await TestUtils.gc();
  } else if (self.gc) {
    // Use --expose_gc for V8 (and Node.js)
    // to pass this flag at chrome launch use: --js-flags="--expose-gc"
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

// For tests which verify that the implementation doesn't do something it shouldn't, it's better not to use a
// timeout. Instead, assume that any reasonable implementation is going to finish work after 2 times around the event
// loop, and use flushAsyncEvents().then(() => assert_array_equals(...));
// Some tests include promise resolutions which may mean the test code takes a couple of event loop visits itself. So go
// around an extra 2 times to avoid complicating those tests.
self.flushAsyncEvents = () => delay(0).then(() => delay(0)).then(() => delay(0)).then(() => delay(0));

self.assert_typed_array_equals = (actual, expected, message) => {
  const prefix = message === undefined ? '' : `${message} `;
  assert_equals(typeof actual, 'object', `${prefix}type is object`);
  assert_equals(actual.constructor, expected.constructor, `${prefix}constructor`);
  assert_equals(actual.byteOffset, expected.byteOffset, `${prefix}byteOffset`);
  assert_equals(actual.byteLength, expected.byteLength, `${prefix}byteLength`);
  assert_equals(actual.buffer.byteLength, expected.buffer.byteLength, `${prefix}buffer.byteLength`);
  assert_array_equals([...actual], [...expected], `${prefix}contents`);
  assert_array_equals([...new Uint8Array(actual.buffer)], [...new Uint8Array(expected.buffer)], `${prefix}buffer contents`);
};

self.makePromiseAndResolveFunc = () => {
  let resolve;
  const promise = new Promise(r => { resolve = r; });
  return [promise, resolve];
};
