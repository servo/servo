// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.DisplayNames.prototype.resolvedOptions
description: >
  Values for the style option
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
  8. Let matcher be ? GetOption(options, "localeMatcher", "string", « "lookup", "best fit" », "best fit").
  ...
  10. Let r be ResolveLocale(%DisplayNames%.[[AvailableLocales]], requestedLocales, opt,
    %DisplayNames%.[[RelevantExtensionKeys]]).
  10. Let style be ? GetOption(options, "style", "string", « "narrow", "short", "long" », "long").
  ...
  12. Let type be ? GetOption(options, "type", "string", « "language", "region", "script", "currency" », undefined).
  13. If type is undefined, throw a TypeError exception.
  ...
  15. Let fallback be ? GetOption(options, "fallback", "string", « "code", "none" », "code").
  ...
  17. Set displayNames.[[Locale]] to the value of r.[[Locale]].
  ...

  CreateDataProperty ( O, P, V )

  ...
  3. Let newDesc be the PropertyDescriptor { [[Value]]: V, [[Writable]]: true, [[Enumerable]]: true,
    [[Configurable]]: true }.
  ...
locale: [en-US]
features: [Intl.DisplayNames]
includes: [propertyHelper.js]
---*/

const styles = ['narrow', 'short', 'long'];
const types = ['language', 'region', 'script', 'currency'];

types.forEach(type => {
  styles.forEach(style => {
    const dn = new Intl.DisplayNames('en-US', { style, type });
    const options = dn.resolvedOptions();

    verifyProperty(options, 'style', {
      value: style,
      writable: true,
      enumerable: true,
      configurable: true
    });

    verifyProperty(options, 'type', {
      value: type,
      writable: true,
      enumerable: true,
      configurable: true
    });

    verifyProperty(options, 'fallback', {
      value: 'code',
      writable: true,
      enumerable: true,
      configurable: true
    });
  });
});
