'use strict';

test(() => {
  assert_true(!!navigator.mediaDevices.setCaptureHandleConfig);
  navigator.mediaDevices.setCaptureHandleConfig({handle: 'X'.repeat(1024)});
}, 'setCaptureHandleConfig does not throw if handle length is 1024.');

test(() => {
  assert_true(!!navigator.mediaDevices.setCaptureHandleConfig);
  assert_throws_js(
      TypeError,
      () => navigator.mediaDevices.setCaptureHandleConfig(
          {handle: 'X'.repeat(1025)}),
      'handle length must be 1024 characters or less');
}, 'setCaptureHandleConfig raises TypeError if handle is invalid.');

test(() => {
  assert_true(!!navigator.mediaDevices.setCaptureHandleConfig);
  navigator.mediaDevices.setCaptureHandleConfig({permittedOrigins: ['*']});
}, 'setCaptureHandleConfig does not throw if asterisk character appears only once.');

test(() => {
  assert_true(!!navigator.mediaDevices.setCaptureHandleConfig);
  assert_throws_dom(
      'NotSupportedError',
      () => navigator.mediaDevices.setCaptureHandleConfig(
          {permittedOrigins: ['*', '*']}),
      'two asterisk characters are not allowed');

  assert_throws_dom(
      'NotSupportedError',
      () => navigator.mediaDevices.setCaptureHandleConfig(
          {permittedOrigins: ['*', 'http://example.com']}),
      'asterisk character is not allowed with valid origins');

  assert_throws_dom(
      'NotSupportedError',
      () => navigator.mediaDevices.setCaptureHandleConfig(
          {permittedOrigins: ['about://blank']}),
      'origins must be valid');
}, 'setCaptureHandleConfig raises NotSupportedError if permittedOrigins is invalid.');


test(() => {
  assert_true(!!navigator.mediaDevices.setCaptureHandleConfig);
  const iframe = document.createElement('iframe');
  document.body.appendChild(iframe);
  const mediaDevices = iframe.contentWindow.navigator.mediaDevices;
  const iframeDOMException = iframe.contentWindow.DOMException;

  assert_throws_dom('InvalidStateError', iframeDOMException, () => {
    mediaDevices.setCaptureHandleConfig();
  });
}, 'setCaptureHandleConfig raises InvalidStateError if not from top-level browsing context.');
