// Copyright 2023 Google Inc.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.locale
description: >
    Checks valid cases for the options argument to the Locale constructor.
info: |
    Intl.Locale( tag [, options] )

    ...
    x. Let fw be ? GetOption(options, "firstDayOfWeek", "string", undefined, undefined).
    x. If fw is not undefined, then
       x. Set fw to !WeekdayToString(fw).
       x. If fw does not match the type sequence (from UTS 35 Unicode Locale Identifier, section 3.2), throw a RangeError exception.
    x. Set opt.[[fw]] to fw.
    ...
    x. Let r be ! ApplyUnicodeExtensionToTag(tag, opt, relevantExtensionKeys).
    ...
    x. Set locale.[[FirstDayOfWeek]] to r.[[fw]].
    ...

features: [Intl.Locale,Intl.Locale-info]
---*/

const validFirstDayOfWeekOptions = [
  ["mon", "en-u-fw-mon"],
  ["tue", "en-u-fw-tue"],
  ["wed", "en-u-fw-wed"],
  ["thu", "en-u-fw-thu"],
  ["fri", "en-u-fw-fri"],
  ["sat", "en-u-fw-sat"],
  ["sun", "en-u-fw-sun"],
  ["1", "en-u-fw-mon"],
  ["2", "en-u-fw-tue"],
  ["3", "en-u-fw-wed"],
  ["4", "en-u-fw-thu"],
  ["5", "en-u-fw-fri"],
  ["6", "en-u-fw-sat"],
  ["7", "en-u-fw-sun"],
  ["0", "en-u-fw-sun"],
  [1, "en-u-fw-mon"],
  [2, "en-u-fw-tue"],
  [3, "en-u-fw-wed"],
  [4, "en-u-fw-thu"],
  [5, "en-u-fw-fri"],
  [6, "en-u-fw-sat"],
  [7, "en-u-fw-sun"],
  [0, "en-u-fw-sun"],
  [true, "en-u-fw"],
  [false, "en-u-fw-false"],
  [null, "en-u-fw-null"],
  ["primidi", "en-u-fw-primidi"],
  ["duodi", "en-u-fw-duodi"],
  ["tridi", "en-u-fw-tridi"],
  ["quartidi", "en-u-fw-quartidi"],
  ["quintidi", "en-u-fw-quintidi"],
  ["sextidi", "en-u-fw-sextidi"],
  ["septidi", "en-u-fw-septidi"],
  ["octidi", "en-u-fw-octidi"],
  ["nonidi", "en-u-fw-nonidi"],
  ["decadi", "en-u-fw-decadi"],
  ["frank", "en-u-fw-frank"],
  ["yungfong", "en-u-fw-yungfong"],
  ["yung-fong", "en-u-fw-yung-fong"],
  ["tang", "en-u-fw-tang"],
  ["frank-yung-fong-tang", "en-u-fw-frank-yung-fong-tang"],
];
for (const [firstDayOfWeek, expected] of validFirstDayOfWeekOptions) {
  assert.sameValue(
    new Intl.Locale('en', { firstDayOfWeek }).toString(),
    expected,
    `new Intl.Locale("en", { firstDayOfWeek: ${firstDayOfWeek} }).toString() returns "${expected}"`
  );
  assert.sameValue(
    new Intl.Locale('en-u-fw-WED', { firstDayOfWeek }).toString(),
    expected,
    `new Intl.Locale("en-u-fw-WED", { firstDayOfWeek: ${firstDayOfWeek} }).toString() returns "${expected}"`
  );
}
