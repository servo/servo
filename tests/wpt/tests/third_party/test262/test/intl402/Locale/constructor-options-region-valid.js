// Copyright 2018 André Bargull; Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.locale
description: >
    Checks error cases for the options argument to the Locale
    constructor.
info: |
    Intl.Locale( tag [, options] )
    10. If options is undefined, then
    11. Else
        a. Let options be ? ToObject(options).
    12. Set tag to ? ApplyOptionsToTag(tag, options).

    ApplyOptionsToTag( tag, options )
    ...
    7. Let region be ? GetOption(options, "region", "string", undefined, undefined).
    ...
    9. If tag matches neither the privateuse nor the grandfathered production, then
      ...
      d. If region is not undefined, then
        i. If tag does not contain a region production, then
          1. Set tag to the concatenation of the language production of tag, the substring corresponding to the "-" script production if present, "-", region, and the rest of tag.
        ii. Else,
          1. Set tag to tag with the substring corresponding to the region production replaced by the string region.

features: [Intl.Locale]
---*/

const validRegionOptions = [
  [undefined, undefined],
  ['FR', 'en-FR'],
  ['554', 'en-NZ'],
  [554, 'en-NZ'],
];
for (const [region, expected] of validRegionOptions) {
  let options = { region };
  let expect = expected || 'en';

  assert.sameValue(
    new Intl.Locale('en', options).toString(),
    expect,
    `new Intl.Locale('en', {region: "${region}"}).toString() returns "${expect}"`
  );

  expect = expected || 'en-US';
  assert.sameValue(
    new Intl.Locale('en-US', options).toString(),
    expect,
    `new Intl.Locale('en-US', {region: "${region}"}).toString() returns "${expect}"`
  );

  expect = (expected || 'en') + '-u-ca-gregory';
  assert.sameValue(
    new Intl.Locale('en-u-ca-gregory', options).toString(),
    expect,
    `new Intl.Locale('en-u-ca-gregory', {region: "${region}"}).toString() returns "${expect}"`
  );

  expect = (expected || 'en-US') + '-u-ca-gregory';
  assert.sameValue(
    new Intl.Locale('en-US-u-ca-gregory', options).toString(),
    expect,
    `new Intl.Locale('en-US-u-ca-gregory', {region: "${region}"}).toString() returns "${expect}"`
  );
}
