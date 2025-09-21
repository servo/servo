'use strict';

test(() => {
  assert_throws_js(TypeError, () => fetchLater());
}, `fetchLater() cannot be called without request.`);

test(() => {
  const result = fetchLater('/');
  assert_false(result.activated, `result.activated should be false for '/'`);
}, `fetchLater() with same-origin (https) URL does not throw.`);

test(() => {
  const url = 'http://localhost';
  const result = fetchLater(url);
  assert_false(result.activated, `result.activated should be false for ${url}`);
}, `fetchLater() with http://localhost URL does not throw.`);

test(() => {
  const url = 'https://localhost';
  const result = fetchLater(url);
  assert_false(result.activated, `result.activated should be false for ${url}`);
}, `fetchLater() with https://localhost URL does not throw.`);

test(() => {
  const url = 'http://127.0.0.1';
  const result = fetchLater(url);
  assert_false(result.activated, `result.activated should be false for ${url}`);
}, `fetchLater() with http://127.0.0.1 URL does not throw.`);

test(() => {
  const url = 'https://127.0.0.1';
  const result = fetchLater(url);
  assert_false(result.activated, `result.activated should be false for ${url}`);
}, `fetchLater() with https://127.0.0.1 URL does not throw.`);

test(() => {
  const url = 'http://[::1]';
  const result = fetchLater(url);
  assert_false(result.activated, `result.activated should be false for ${url}`);
}, `fetchLater() with http://[::1] URL does not throw.`);

test(() => {
  const url = 'https://[::1]';
  const result = fetchLater(url);
  assert_false(result.activated, `result.activated should be false for ${url}`);
}, `fetchLater() with https://[::1] URL does not throw.`);

test(() => {
  const url = 'https://example.com';
  const result = fetchLater(url);
  assert_false(result.activated, `result.activated should be false for ${url}`);
}, `fetchLater() with https://example.com URL does not throw.`);

test(() => {
  const httpUrl = 'http://example.com';
  assert_throws_dom(
      'SecurityError', () => fetchLater(httpUrl),
      `should throw SecurityError for insecure http url ${httpUrl}`);
}, `fetchLater() throws SecurityError on non-trustworthy http URL.`);

test(() => {
  assert_throws_js(TypeError, () => fetchLater('file://tmp'));
}, `fetchLater() throws TypeError on file:// scheme.`);

test(() => {
  assert_throws_js(TypeError, () => fetchLater('ftp://example.com'));
}, `fetchLater() throws TypeError on ftp:// scheme.`);

test(() => {
  assert_throws_js(TypeError, () => fetchLater('ssh://example.com'));
}, `fetchLater() throws TypeError on ssh:// scheme.`);

test(() => {
  assert_throws_js(TypeError, () => fetchLater('wss://example.com'));
}, `fetchLater() throws TypeError on wss:// scheme.`);

test(() => {
  assert_throws_js(TypeError, () => fetchLater('about:blank'));
}, `fetchLater() throws TypeError on about: scheme.`);

test(() => {
  assert_throws_js(TypeError, () => fetchLater(`javascript:alert('');`));
}, `fetchLater() throws TypeError on javascript: scheme.`);

test(() => {
  assert_throws_js(TypeError, () => fetchLater('data:text/plain,Hello'));
}, `fetchLater() throws TypeError on data: scheme.`);

test(() => {
  assert_throws_js(
      TypeError, () => fetchLater('blob:https://example.com/some-uuid'));
}, `fetchLater() throws TypeError on blob: scheme.`);

test(() => {
  assert_throws_js(
      RangeError,
      () => fetchLater('https://www.google.com', {activateAfter: -1}));
}, `fetchLater() throws RangeError on negative activateAfter.`);

test(() => {
  const result = fetchLater('/');
  assert_false(result.activated);
}, `fetchLater()'s return tells the deferred request is not yet sent.`);

test(() => {
  const result = fetchLater('/');
  assert_throws_js(TypeError, () => result.activated = true);
}, `fetchLater() throws TypeError when mutating its returned state.`);

test(() => {
  const controller = new AbortController();
  // Immediately aborts the controller.
  controller.abort();
  assert_throws_dom(
      'AbortError', () => fetchLater('/', {signal: controller.signal}));
}, `fetchLater() throws AbortError when its initial abort signal is aborted.`);

test(() => {
  const controller = new AbortController();
  const result = fetchLater('/', {signal: controller.signal});
  assert_false(result.activated);
  controller.abort();
  assert_false(result.activated);
}, `fetchLater() does not throw error when it is aborted before sending.`);
