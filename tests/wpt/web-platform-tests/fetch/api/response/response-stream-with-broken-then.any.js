// META: script=../resources/utils.js

promise_test(async () => {
  add_completion_callback(() => delete Object.prototype.then);
  const hello = new TextEncoder().encode('hello');
  const bye = new TextEncoder().encode('bye');
  const rs = new ReadableStream({
    start(controller) {
      controller.enqueue(hello);
      controller.close();
    }
  });
  const resp = new Response(rs);
  Object.prototype.then = (onFulfilled) => {
    delete Object.prototype.then;
    onFulfilled({done: false, value: bye});
  };
  const text = await resp.text();
  assert_equals(text, 'bye', 'The valud should be replaced with "bye".');
}, 'Inject {done: false, value: bye} via Object.prototype.then.');

promise_test(async (t) => {
  add_completion_callback(() => delete Object.prototype.then);
  const hello = new TextEncoder().encode('hello');
  const rs = new ReadableStream({
    start(controller) {
      controller.enqueue(hello);
      controller.close();
    }
  });
  const resp = new Response(rs);
  Object.prototype.then = (onFulfilled) => {
    delete Object.prototype.then;
    onFulfilled({done: false, value: undefined});
  };
  promise_rejects(t, TypeError(), resp.text(),
      'The value should be replaced with undefined.');
}, 'Inject {done: false, value: undefined} via Object.prototype.then.');

promise_test(async (t) => {
  add_completion_callback(() => delete Object.prototype.then);
  const hello = new TextEncoder().encode('hello');
  const rs = new ReadableStream({
    start(controller) {
      controller.enqueue(hello);
      controller.close();
    }
  });
  const resp = new Response(rs);
  Object.prototype.then = (onFulfilled) => {
    delete Object.prototype.then;
    onFulfilled(undefined);
  };
  promise_rejects(t, TypeError(), resp.text(),
      'The read result should be replaced with undefined.');
}, 'Inject undefined via Object.prototype.then.');

promise_test(async (t) => {
  add_completion_callback(() => delete Object.prototype.then);
  const hello = new TextEncoder().encode('hello');
  const rs = new ReadableStream({
    start(controller) {
      controller.enqueue(hello);
      controller.close();
    }
  });
  const resp = new Response(rs);
  Object.prototype.then = (onFulfilled) => {
    delete Object.prototype.then;
    onFulfilled(8.2);
  };
  promise_rejects(t, TypeError(), resp.text(),
      'The read result should be replaced with a number.');
}, 'Inject 8.2 via Object.prototype.then.');

