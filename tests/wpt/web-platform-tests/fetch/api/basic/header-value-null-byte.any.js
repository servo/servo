// META: global=window,worker

promise_test(t => {
  return promise_rejects(t, new TypeError(), fetch("../../../xhr/resources/parse-headers.py?my-custom-header="+encodeURIComponent("x\0x")));
}, "Ensure fetch() rejects null bytes in headers");
