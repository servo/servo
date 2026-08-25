// Copyright 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-formatdatetimepattern
description: |
  Checks that numberingSystem property used correctly.
info: >
  11.5.5 FormatDateTimePattern ( dateTimeFormat, format, pattern, epochNanoseconds )
    ...
    3. Perform ! CreateDataPropertyOrThrow(nfOptions, "numberingSystem", dateTimeFormat.[[NumberingSystem]]).
    ...
    8. Perform ! CreateDataPropertyOrThrow(nf2Options, "numberingSystem", dateTimeFormat.[[NumberingSystem]]).
    ...
    11. If format has a field [[fractionalSecondDigits]], then
    ...
      d. Perform ! CreateDataPropertyOrThrow(nf3Options, "numberingSystem", dateTimeFormat.[[NumberingSystem]]).

locale: [en-US, en-US-u-nu-arab, en-US-u-nu-deva, en-US-u-nu-hanidec]
---*/

const localesAndResults = [
  [ "en-US", "2:35:06", "2:35:06.789", "02:35:06", "6" ],
  [ "en-US-u-nu-arab", "٢:٣٥:٠٦", "٢:٣٥:٠٦٫٧٨٩", "٠٢:٣٥:٠٦", "٦" ],
  [ "en-US-u-nu-deva", "२:३५:०६", "२:३५:०६.७८९", "०२:३५:०६", "६" ],
  [ "en-US-u-nu-hanidec", "二:三五:〇六", "二:三五:〇六.七八九", "〇二:三五:〇六 AM", "六" ],
]
const time = new Date(2024, 0, 1, 2, 35, 6, 789);

for (const [locale, expectedNoFractional, expectedFractional, expectedTwoDigit, expectedAllNumeric] of localesAndResults) {
  const formattedNoFractional = new Intl.DateTimeFormat(locale, {
      hour: "numeric",
      minute: "numeric",
      second: "numeric",
    }).format(time);

  const formattedFractional = new Intl.DateTimeFormat(locale, {
      hour: "numeric",
      minute: "numeric",
      second: "numeric",
      fractionalSecondDigits: 3,
    }).format(time);

  const formattedTwoDigit = new Intl.DateTimeFormat(locale, {
      hour: "2-digit",
      minute: "2-digit",
      second: "2-digit",
    }).format(time);

  const formattedAllNumeric = new Intl.DateTimeFormat(locale, {
      second: "numeric",
    }).format(time);

  assert.sameValue(formattedNoFractional.includes(expectedNoFractional), true, `${locale}: display without fractionalSecondDigits`);
  assert.sameValue(formattedFractional.includes(expectedFractional), true, `${locale}: display with fractionalSecondDigits`);
  assert.sameValue(formattedTwoDigit.includes(expectedTwoDigit), true, `${locale}: display all time units in 2-digit`);
  assert.sameValue(formattedAllNumeric.includes(expectedAllNumeric), true,`${locale}: display one numeric-styled unit`);
}

