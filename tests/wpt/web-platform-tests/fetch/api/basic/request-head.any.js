promise_test(function(test) {
  var requestInit = {"method": "HEAD", "body": "test"};
  return promise_rejects(test, new TypeError(), fetch(".", requestInit));
}, "Fetch with HEAD with body");
