promise_test((test) => {
    return fetch("resources/bad-gzip-body.py").then(res => {
      assert_equals(res.status, 200);
    });
}, "Fetching a resource with bad gzip content should still resolve");

[
  "arrayBuffer",
  "blob",
  "formData",
  "json",
  "text"
].forEach(method => {
  promise_test(t => {
    return fetch("resources/bad-gzip-body.py").then(res => {
      assert_equals(res.status, 200);
      return promise_rejects(t, new TypeError(), res[method]());
    });
  }, "Consuming the body of a resource with bad gzip content with " + method + "() should reject");
});
