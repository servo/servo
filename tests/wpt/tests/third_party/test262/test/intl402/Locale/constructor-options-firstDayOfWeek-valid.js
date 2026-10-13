// Copyright 2023 Google Inc.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.locale
description: >
    Checks valid cases for the options argument to the Locale constructor.
info: |
    Intl.Locale( tag [, options] )

    ...
    22. Let fw be ? GetOption(options, "firstDayOfWeek", string, empty, undefined).
    23. If fw is not undefined, then
       a. Set fw to WeekdayToUValue(fw).
       b. If fw cannot be matched by the type Unicode locale nonterminal, throw a RangeError exception.
    24. Set opt.[[fw]] to fw.
    ...
    35. Let r be MakeLocaleRecord(tag, opt, localeExtensionKeys).
    ...
    39. Set locale.[[FirstDayOfWeek]] to r.[[fw]].
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
  [-0, "en-u-fw-sun"],
  [100, "en-u-fw-100"],
  [1000, "en-u-fw-1000"],
  [1n, "en-u-fw-mon"],
  [2n, "en-u-fw-tue"],
  [3n, "en-u-fw-wed"],
  [4n, "en-u-fw-thu"],
  [5n, "en-u-fw-fri"],
  [6n, "en-u-fw-sat"],
  [7n, "en-u-fw-sun"],
  [0n, "en-u-fw-sun"],
  [100n, "en-u-fw-100"],
  [1000n, "en-u-fw-1000"],
  [NaN, "en-u-fw-nan"],
  [Infinity, "en-u-fw-infinity"],
  [true, "en-u-fw"],
  [false, "en-u-fw-false"],
  [null, "en-u-fw-null"],
  ["yes", "en-u-fw-yes"],
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
    new Intl.Locale("en", { firstDayOfWeek }).toString(),
    expected,
    `new Intl.Locale("en", { firstDayOfWeek: ${firstDayOfWeek} }).toString() returns "${expected}"`
  );
  assert.sameValue(
    new Intl.Locale("en-u-fw-WED", { firstDayOfWeek }).toString(),
    expected,
    `new Intl.Locale("en-u-fw-WED", { firstDayOfWeek: ${firstDayOfWeek} }).toString() returns "${expected}"`
  );

  let upperCase = String(firstDayOfWeek).toUpperCase();
  assert.sameValue(
    new Intl.Locale("en", { firstDayOfWeek: upperCase }).toString(),
    expected,
    `new Intl.Locale("en", { firstDayOfWeek: ${upperCase} }).toString() returns "${expected}"`
  );
}
