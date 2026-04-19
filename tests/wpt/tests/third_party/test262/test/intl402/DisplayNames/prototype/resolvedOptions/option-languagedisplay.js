// Copyright (C) 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-Intl.DisplayNames.prototype.resolvedOptions
description: Values for the languageDisplay option
info: |
  Intl.DisplayNames.prototype.resolvedOptions ()

  1. Let pr be the this value.
  2. If Type(pr) is not Object or pr does not have an [[InitializedDisplayNames]] internal slot,
    throw a TypeError exception.
  3. Let options be ! ObjectCreate(%ObjectPrototype%).
  4. For each row of Table 6, except the header row, in table order, do
    a. Let p be the Property value of the current row.
    b. Let v be the value of pr's internal slot whose name is the Internal Slot value of the current row.
    c. If v is not undefined, then
      i. Perform ! CreateDataPropertyOrThrow(options, p, v).
  6. Return options.

  Table 6: Resolved Options of DisplayNames Instances

  [[Locale]]: "locale"
  [[Style]]: "style"
  [[Type]]: "type"
  [[Fallback]]: "fallback"
  [[LanguageDisplay]]: "languageDisplay"

  Intl.DisplayNames ( locales , options )

  ...
  24. Let languageDisplay be ? GetOption(options, "languageDisplay", "string",
      « "dialect", "standard" », "dialect").
  25. If type is "language", then
      a. Set displayNames.[[LanguageDisplay]] to languageDisplay.
      b. Let typeFields be typeFields.[[<languageDisplay>]].
      c. Assert: typeFields is a Record (see 1.4.3).
  ...

  CreateDataProperty ( O, P, V )

  ...
  3. Let newDesc be the PropertyDescriptor { [[Value]]: V, [[Writable]]: true, [[Enumerable]]: true,
    [[Configurable]]: true }.
  ...
locale: [en-US]
features: [Intl.DisplayNames-v2]
includes: [propertyHelper.js]
---*/

var dn;

dn = new Intl.DisplayNames('en-US', { type: 'language', languageDisplay: 'dialect' });

verifyProperty(dn.resolvedOptions(), 'languageDisplay', {
  value: 'dialect',
  writable: true,
  enumerable: true,
  configurable: true
});

dn = new Intl.DisplayNames('en-US', { type: 'language', languageDisplay: 'standard' });

verifyProperty(dn.resolvedOptions(), 'languageDisplay', {
  value: 'standard',
  writable: true,
  enumerable: true,
  configurable: true
});
