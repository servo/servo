// Copyright 2018 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.locale
description: >
    Checks valid cases for the options argument to the Locale constructor.
info: |
    Intl.Locale( tag [, options] )

    ...
    20. Let hc be ? GetOption(options, "hourCycle", "string", « "h11", "h12", "h23", "h24" », undefined).
    21. Set opt.[[hc]] to hc.
    ...
    30. Let r be ! ApplyUnicodeExtensionToTag(tag, opt, relevantExtensionKeys).
    ...

    ApplyUnicodeExtensionToTag( tag, options, relevantExtensionKeys )

    ...
    8. Let locale be the String value that is tag with all Unicode locale extension sequences removed.
    9. Let newExtension be ! CanonicalizeUnicodeExtension(attributes, keywords).
    10. If newExtension is not the empty String, then
        a. Let locale be ! InsertUnicodeExtension(locale, newExtension).
    ...

    CanonicalizeUnicodeExtension( attributes, keywords )
    ...
    4. Repeat for each element entry of keywords in List order,
        a. Let keyword be entry.[[Key]].
        b. If entry.[[Value]] is not the empty String, then
            i. Let keyword be the string-concatenation of keyword, "-", and entry.[[Value]].
        c. Append keyword to fullKeywords.
    ...
features: [Intl.Locale]
---*/

const validHourCycleOptions = [
  'h11',
  'h12',
  'h23',
  'h24',
  { toString() { return 'h24'; } },
];
for (const hourCycle of validHourCycleOptions) {
  const expected = String(hourCycle);
  let expect = 'en-u-hc-' + expected;

  assert.sameValue(
    new Intl.Locale('en', {hourCycle}).toString(),
    expect,
    `new Intl.Locale("en", {hourCycle: "${hourCycle}"}).toString() returns "${expect}"`
  );

  assert.sameValue(
    new Intl.Locale('en-u-hc-h00', {hourCycle}).toString(),
    expect,
    `new Intl.Locale("en-u-hc-h00", {hourCycle: "${hourCycle}"}).toString() returns "${expect}"`
  );

  assert.sameValue(
    new Intl.Locale('en-u-hc-h12', {hourCycle}).toString(),
    expect,
    `new Intl.Locale("en-u-hc-h12", {hourCycle: "${hourCycle}"}).toString() returns "${expect}"`
  );

  assert.sameValue(
    new Intl.Locale('en-u-hc-h00', {hourCycle}).hourCycle,
    expected,
    `new Intl.Locale("en-u-hc-h00", {hourCycle: "${hourCycle}"}).hourCycle equals "${expected}"`
  );
}
