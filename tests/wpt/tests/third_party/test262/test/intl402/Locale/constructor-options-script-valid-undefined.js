// Copyright 2018 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.locale
description: >
    Verify valid script option values (undefined)
info: |
    Intl.Locale( tag [, options] )
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

assert.sameValue(
  new Intl.Locale('en', {script: undefined}).toString(),
  'en',
  `new Intl.Locale('en', {script: undefined}).toString() returns "en"`
);

assert.sameValue(
  new Intl.Locale('en-DK', {script: undefined}).toString(),
  'en-DK',
  `new Intl.Locale('en-DK', {script: undefined}).toString() returns "en-DK"`
);

assert.sameValue(
  new Intl.Locale('en-Cyrl', {script: undefined}).toString(),
  'en-Cyrl',
  `new Intl.Locale('en-Cyrl', {script: undefined}).toString() returns "en-Cyrl"`
);

