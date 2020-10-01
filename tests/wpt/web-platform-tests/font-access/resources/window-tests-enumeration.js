'use strict';

font_access_test(async t => {
  const iterator = navigator.fonts.query();

  if (!isPlatformSupported()) {
    await promise_rejects_dom(t, 'NotSupportedError', (async () => {
      for await (const f of iterator) {
      }
    })());
    return;
  }

  assert_equals(typeof iterator, 'object', 'query() should return an Object');
  assert_true(!!iterator[Symbol.asyncIterator],
              'query() has an asyncIterator method');

  const availableFonts = [];
  for await (const f of iterator) {
    availableFonts.push(f);
  }

  assert_fonts_exist(availableFonts, getEnumerationTestSet());
}, 'query(): standard fonts returned');
