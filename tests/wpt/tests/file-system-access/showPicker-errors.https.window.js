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
        self[showPickerMethod]({types: [{accept: {'': ['.foo']}}]}));
    await promise_rejects_js(
        t, TypeError,
        self[showPickerMethod]({types: [{accept: {'  ': ['.foo']}}]}));
  }, showPickerMethod + ': MIME type can\'t be an empty string.');

  promise_test(async t => {
    await promise_rejects_js(
        t, TypeError,
        self[showPickerMethod]({types: [{accept: {'image': ['.foo']}}]}));
  }, showPickerMethod + ': MIME type must have subtype.');

  promise_test(async t => {
    await promise_rejects_js(
        t, TypeError,
        self[showPickerMethod]({types: [{accept: {'  /plain': ['.foo']}}]}));
  }, showPickerMethod + ': MIME type can\'t have empty type.');

  promise_test(async t => {
    await promise_rejects_js(
        t, TypeError,
        self[showPickerMethod]({types: [{accept: {'image/  ': ['.foo']}}]}));
  }, showPickerMethod + ': MIME type can\'t have empty subtype.');

  promise_test(async t => {
    await promise_rejects_js(
        t, TypeError,
        self[showPickerMethod](
            {types: [{accept: {'text/plain;charset=utf8': ['.txt']}}]}));
  }, showPickerMethod + ': MIME type can\'t have parameters.');

  promise_test(async t => {
    await promise_rejects_js(t, TypeError, self[showPickerMethod]({
                               types: [{accept: {'text>foo/plain': ['.txt']}}]
                             }));
  }, showPickerMethod + ': MIME type can\'t have invalid characters in type.');

  promise_test(async t => {
    await promise_rejects_js(
        t, TypeError,
        self[showPickerMethod]({types: [{accept: {'text / plain': ['.txt']}}]}));
  }, showPickerMethod + ': MIME type can\'t have whitespace in the middle.');

  promise_test(
      async t => {
        await promise_rejects_js(
            t, TypeError,
            self[showPickerMethod](
                {types: [{accept: {'text/plain>foo': ['.txt']}}]}));
      },
      showPickerMethod +
          ': MIME type can\'t have invalid characters in subtype.');

  promise_test(async t => {
    await promise_rejects_js(t, TypeError, self[showPickerMethod]({
      startIn: 'secrets',
    }));
  }, showPickerMethod + ': unknown well-known starting directory.');

  promise_test(async t => {
    await promise_rejects_js(t, TypeError, self[showPickerMethod]({
                               startIn: '',
                             }));
  }, showPickerMethod + ': starting directory can\t be empty.');

  promise_test(async t => {
    await promise_rejects_js(t, TypeError, self[showPickerMethod]({
                               startIn: null,
                             }));
  }, showPickerMethod + ': starting directory can\t be null.');

  promise_test(async t => {
    await promise_rejects_js(t, TypeError, self[showPickerMethod]({
      id: "inv*l:d\\ chara<ters",
    }));
  }, showPickerMethod + ': starting directory ID contains invalid characters.');

  promise_test(async t => {
    await promise_rejects_js(t, TypeError, self[showPickerMethod]({
      id: "id-length-cannot-exceed-32-characters",
    }));
  }, showPickerMethod + ': starting directory ID cannot exceed 32 characters.');

  const invalid_extensions = {
    '.extensiontoolong': 'extension length more than 16.',
    '.txt.': 'extenstion ends with "."',
    'txt': 'extenstion does not start with "."',
    '.$txt' : 'illegal character "$"',
    '.t<xt': 'illegal character "<"',
    '.t/xt': 'illegal character "\"',
    '.\txt': 'illegal character "/"',
    '.txt\\': 'illegal characters "\\"',
    '.txt?': 'illegal character "?"',
    '.txt*': 'illegal character "*"',
    '.{txt': 'illegal character "{"',
    '.}txt': 'illegal character "}"',
    ' .txt': 'illegal whitespace at front of extension',
    '. txt': 'illegal whitespace in extension',
    '.txt ': 'illegal whitespace at end of extension',
    '.\u202etxt\u202e' : 'illegal RTL character',
    '.t\u00E6xt': 'non-ASCII character "æ"',
    '.קום': 'non-ASCII character "קום"',
    '.txt🙂': 'non-ASCII character "🙂"',
    '.{txt}': 'illegal characters "{" and "}"',
  }

  for (const [extension, description] of Object.entries(invalid_extensions)) {
    define_file_picker_extension_error_test(showPickerMethod, extension, description)
  }
}

function define_file_picker_extension_error_test(showPickerMethod, extension, description) {
  promise_test(async t => {
    await promise_rejects_js(
      t, TypeError,
      self[showPickerMethod](
        { types: [{ accept: { 'text/plain': ['.txt', extension] } }] }));
  }, showPickerMethod + ': invalid extension "' + extension + '". ' + description + ".");
}