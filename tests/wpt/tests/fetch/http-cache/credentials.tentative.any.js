// META: global=window,worker
// META: title=HTTP Cache - Content
// META: timeout=long
// META: script=/common/utils.js
// META: script=http-cache.js

// This is a tentative test.
// Firefox behavior is used as expectations.
//
// whatwg/fetch issue:
// https://github.com/whatwg/fetch/issues/1253
//
// Chrome design doc:
// https://docs.google.com/document/d/1lvbiy4n-GM5I56Ncw304sgvY5Td32R6KHitjRXvkZ6U/edit#

const request_cacheable = {
  request_headers: [],
  response_headers: [
    ['Cache-Control', 'max-age=3600'],
  ],
  // TODO(arthursonzogni): The behavior is tested only for same-origin requests.
  // It must behave similarly for cross-site and cross-origin requests. The
  // problems is the http-cache.js infrastructure returns the
  // "Server-Request-Count" as HTTP response headers, which aren't readable for
  // CORS requests.
  base_url: location.href.replace(/\/[^\/]*$/, '/'),
};

const request_credentialled = { ...request_cacheable, credentials: 'include', };
const request_anonymous     = { ...request_cacheable, credentials: 'omit', };

const responseIndex = count => {
  return {
    expected_response_headers: [
      ['Server-Request-Count', count.toString()],
    ],
  }
};

var tests = [
  {
    name: 'same-origin: 2xAnonymous, 2xCredentialled, 1xAnonymous',
    requests: [
      { ...request_anonymous     , ...responseIndex(1)} ,
      { ...request_anonymous     , ...responseIndex(1)} ,
      { ...request_credentialled , ...responseIndex(2)} ,
      { ...request_credentialled , ...responseIndex(2)} ,
      { ...request_anonymous     , ...responseIndex(1)} ,
    ]
  },
  {
    name: 'same-origin: 2xCredentialled, 2xAnonymous, 1xCredentialled',
    requests: [
      { ...request_credentialled , ...responseIndex(1)} ,
      { ...request_credentialled , ...responseIndex(1)} ,
      { ...request_anonymous     , ...responseIndex(2)} ,
      { ...request_anonymous     , ...responseIndex(2)} ,
      { ...request_credentialled , ...responseIndex(1)} ,
    ]
  },
];
run_tests(tests);
