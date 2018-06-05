importScripts("/resources/testharness.js");

var blob, readerSync;
setup(function() {
  readerSync = new FileReaderSync();
  blob = new Blob(["test"]);
});

test(function() {
  assert_true(readerSync instanceof FileReaderSync);
}, "Interface");

test(function() {
  var text = readerSync.readAsText(blob);
  assert_equals(text, "test");
}, "readAsText");

test(function() {
  var data = readerSync.readAsDataURL(blob);
  assert_equals(data.indexOf("data:"), 0);
}, "readAsDataURL");

test(function() {
  var data = readerSync.readAsArrayBuffer(blob);
  assert_true(data instanceof ArrayBuffer);
}, "readAsArrayBuffer");

done();
