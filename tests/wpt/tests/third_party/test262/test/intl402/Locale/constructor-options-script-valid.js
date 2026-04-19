// Copyright 2018 André Bargull; Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.locale
description: >
    Verify valid language option values (various)
info: |
    Intl.Locale( tag [, options] )
    9. Else,
        a. Let tag be ? ToString(tag).
    10. If options is undefined, then
    11. Else
        a. Let options be ? ToObject(options).
    12. Set tag to ? ApplyOptionsToTag(tag, options).

    ApplyOptionsToTag( tag, options )
    ...
    5. Let script be ? GetOption(options, "script", "string", undefined, undefined).
    ...
    9. If tag matches neither the privateuse nor the grandfathered production, then
      ...
      c. If script is not undefined, then
        i. If tag does not contain a script production, then
          1. Set tag to the concatenation of the language production of tag, "-", script, and the rest of tag.
        ii. Else,
          1. Set tag to tag with the substring corresponding to the script production replaced by the string script.


features: [Intl.Locale]
---*/

const validScriptOptions = [
  [null, 'Null'],
  ['bali', 'Bali'],
  ['Bali', 'Bali'],
  ['bALI', 'Bali'],
  [{ toString() { return 'Brai' } }, 'Brai'],
];
for (const [script, expected] of validScriptOptions) {
  let expect = expected ? 'en-' + expected : 'en';

  assert.sameValue(
    new Intl.Locale('en', { script }).toString(),
    expect,
    `new Intl.Locale("en", {script: "${script}"}).toString() returns "${expect}"`
  );

  expect = (expected ? ('en-' + expected) : 'en') + '-DK';
  assert.sameValue(
    new Intl.Locale('en-DK', { script }).toString(),
    expect,
    `new Intl.Locale("en-DK", {script: "${script}"}).toString() returns "${expect}"`
  );

  expect = expected ? ('en-' + expected) : 'en-Cyrl';
  assert.sameValue(
    new Intl.Locale('en-Cyrl', { script }).toString(),
    expect,
    `new Intl.Locale("en-Cyrl", {script: "${script}"}).toString() returns "${expect}"`
  );
}
