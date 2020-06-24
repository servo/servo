// META: script=resources/test-helpers.js

promise_test(async t => {
  await promise_rejects_dom(t, 'SecurityError', self.showOpenFilePicker());
}, 'showOpenFilePicker: Showing a picker requires user activation.');

promise_test(async t => {
  await promise_rejects_dom(t, 'SecurityError', self.showSaveFilePicker());
}, 'showSaveFilePicker: Showing a picker requires user activation.');

promise_test(async t => {
  await promise_rejects_dom(t, 'SecurityError', self.showDirectoryPicker());
}, 'showDirectoryPicker: Showing a picker requires user activation.');

// TODO(mek): Add tests for cross-origin iframes, opaque origins, etc.

define_file_picker_error_tests('showOpenFilePicker');
define_file_picker_error_tests('showSaveFilePicker');

function define_file_picker_error_tests(showPickerMethod) {
  promise_test(async t => {
    await promise_rejects_js(
        t, TypeError,
        self[showPickerMethod]({excludeAcceptAllOption: true, types: []}));
  }, showPickerMethod + ': File picker requires at least one accepted type.');

  promise_test(async t => {
    await promise_rejects_js(
        t, TypeError,
        self[showPickerMethod]({types: [{accept: {'': ['foo']}}]}));
    await promise_rejects_js(
        t, TypeError,
        self[showPickerMethod]({types: [{accept: {'  ': ['foo']}}]}));
  }, showPickerMethod + ': MIME type can\'t be an empty string.');

  promise_test(async t => {
    await promise_rejects_js(
        t, TypeError,
        self[showPickerMethod]({types: [{accept: {'image': ['foo']}}]}));
  }, showPickerMethod + ': MIME type must have subtype.');

  promise_test(async t => {
    await promise_rejects_js(
        t, TypeError,
        self[showPickerMethod]({types: [{accept: {'  /plain': ['foo']}}]}));
  }, showPickerMethod + ': MIME type can\'t have empty type.');

  promise_test(async t => {
    await promise_rejects_js(
        t, TypeError,
        self[showPickerMethod]({types: [{accept: {'image/  ': ['foo']}}]}));
  }, showPickerMethod + ': MIME type can\'t have empty subtype.');

  promise_test(async t => {
    await promise_rejects_js(
        t, TypeError,
        self[showPickerMethod](
            {types: [{accept: {'text/plain;charset=utf8': ['txt']}}]}));
  }, showPickerMethod + ': MIME type can\'t have parameters.');

  promise_test(async t => {
    await promise_rejects_js(t, TypeError, self[showPickerMethod]({
                               types: [{accept: {'text>foo/plain': ['txt']}}]
                             }));
  }, showPickerMethod + ': MIME type can\'t have invalid characters in type.');

  promise_test(async t => {
    await promise_rejects_js(
        t, TypeError,
        self[showPickerMethod]({types: [{accept: {'text / plain': ['txt']}}]}));
  }, showPickerMethod + ': MIME type can\'t have whitespace in the middle.');

  promise_test(
      async t => {
        await promise_rejects_js(
            t, TypeError,
            self[showPickerMethod](
                {types: [{accept: {'text/plain>foo': ['txt']}}]}));
      },
      showPickerMethod +
          ': MIME type can\'t have invalid characters in subtype.');
}