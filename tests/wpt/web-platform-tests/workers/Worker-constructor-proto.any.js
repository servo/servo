//META: global=!default, worker
test(() => {
  proto = new Number(42)
  assert_equals(String(Object.getPrototypeOf(WorkerLocation)).replace(/\n/g, " ").replace(/\s\s+/g, " "), "function () { [native code] }");
  WorkerLocation.__proto__ = proto;
  assert_object_equals(Object.getPrototypeOf(WorkerLocation), Object(42));
}, 'Tests that setting the proto of a built in constructor is not reset.');
