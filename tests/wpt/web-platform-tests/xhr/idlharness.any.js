// META: global=window,dedicatedworker,sharedworker
// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js
// META: timeout=long

idl_test(
  ['xhr'],
  ['dom', 'html'],
  idl_array => {
    idl_array.add_objects({
      XMLHttpRequest: ['new XMLHttpRequest()'],
      XMLHttpRequestUpload: ['(new XMLHttpRequest()).upload'],
      FormData: ['new FormData()'],
      ProgressEvent: ['new ProgressEvent("type")'],
    });
    if (self.Window) {
      self.form = document.createElement('form');
      idl_array.add_objects({ FormData: ['new FormData(form)'] });
    }
  }
);
