'use strict';

font_access_test(async t => {
  await promise_rejects_dom(
      t, 'NotSupportedError', navigator.fonts.showFontChooser());
  await promise_rejects_dom(
      t, 'NotSupportedError', navigator.fonts.showFontChooser({all: false}));
});
