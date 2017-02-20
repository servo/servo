importScripts('/resources/testharness.js');

test(t => {
  assert_throws('NetworkError', () => {
    importScripts("nosiniff-error-worker.py");
  });
}, "importScripts throws on 'nosniff' violation");

done();
