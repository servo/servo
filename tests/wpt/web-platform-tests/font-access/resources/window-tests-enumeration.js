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

font_access_test(async t => {
  const iterator = navigator.fonts.query();

  if (!isPlatformSupported()) {
    await promise_rejects_dom(t, 'NotSupportedError', (async () => {
                                for await (const f of iterator) {
                                }
                              })());
    return;
  }

  // The following tests that fonts are sorted. Postscript names are expected to
  // be encoded in a subset of the ASCII character set.
  // See: https://docs.microsoft.com/en-us/typography/opentype/spec/name
  // Should the Postscript name contain characters that are multi-byte, this
  // test may erroneously fail.
  let previousFont = null;
  for await (const font of iterator) {
    if (previousFont) {
      assert_true(
          previousFont.postscriptName < font.postscriptName,
          `font is not in expected order. expected: ${
              previousFont.postscriptName} < ${font.postscriptName}`);
    }

    previousFont = font;
  }
}, 'query(): fonts are sorted');
