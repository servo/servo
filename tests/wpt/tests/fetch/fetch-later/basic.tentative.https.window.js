'use strict';

test(() => {
  assert_throws_js(TypeError, () => fetchLater());
}, `fetchLater() cannot be called without request.`);

test(() => {
  assert_throws_js(TypeError, () => fetchLater('http://www.google.com'));
  assert_throws_js(TypeError, () => fetchLater('file://tmp'));
  assert_throws_js(TypeError, () => fetchLater('ssh://example.com'));
  assert_throws_js(TypeError, () => fetchLater('wss://example.com'));
  assert_throws_js(TypeError, () => fetchLater('about:blank'));
  assert_throws_js(TypeError, () => fetchLater(`javascript:alert('');`));
}, `fetchLater() throws TypeError on non-HTTPS URL.`);

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
