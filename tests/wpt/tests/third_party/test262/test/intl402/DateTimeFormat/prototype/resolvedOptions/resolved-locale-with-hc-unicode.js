// Copyright 2018 André Bargull. All rights reserved.
// This code is governed by the license found in the LICENSE file.

/*---
esid: sec-Intl.DateTimeFormat.prototype.resolvedOptions
description: >
  The resolved locale doesn't include a hc Unicode extension value if the
  hour12 or hourCycle option is also present.
info: |
  11.1.2 CreateDateTimeFormat( dateTimeFormat, locales, options, required, defaults )
    ...
    13. Let hour12 be ? GetOption(options, "hour12", boolean, empty, undefined).
    14. Let hourCycle be ? GetOption(options, "hourCycle", string, « "h11", "h12", "h23", "h24" », undefined).
    15. If hour12 is not undefined, then
      a. Set hourCycle to null.
    ...

  9.2.6 ResolveLocale(availableLocales, requestedLocales, options, relevantExtensionKeys, localeData)
    ...
    8. For each element key of relevantExtensionKeys in List order, do
      ...
      i. If options has a field [[<key>]], then
        i. Let optionsValue be options.[[<key>]].
        ii. Assert: Type(optionsValue) is either String, Undefined, or Null.
        iii. If keyLocaleData contains optionsValue, then
          1. If SameValue(optionsValue, value) is false, then
            a. Let value be optionsValue.
            b. Let supportedExtensionAddition be "".
      ...
---*/

var defaultLocale = new Intl.DateTimeFormat().resolvedOptions().locale;
var defaultLocaleWithHourCycle = defaultLocale + "-u-hc-h11";

function assertLocale(locale, expectedLocale, options, message) {
  var resolved = new Intl.DateTimeFormat(locale, {
    hour: "2-digit",
    hour12: options.hour12,
    hourCycle: options.hourCycle,
  }).resolvedOptions();
  assert.sameValue(resolved.locale, expectedLocale, message + " (With hour option.)");

  // Also test the case when no hour option is present at all.
  // The resolved options don't include hour12 and hourCycle if the date-time
  // formatter doesn't include an hour option. This restriction doesn't apply
  // to the hc Unicode extension value.
  resolved = new Intl.DateTimeFormat(locale, {
    hour12: options.hour12,
    hourCycle: options.hourCycle,
  }).resolvedOptions();
  assert.sameValue(resolved.locale, expectedLocale, message + " (Without hour option.)");
}

assertLocale(defaultLocaleWithHourCycle, defaultLocale, {
  hour12: false,
  hourCycle: "h23",
}, "hour12 and hourCycle options and hc Unicode extension value are present.");

assertLocale(defaultLocaleWithHourCycle, defaultLocale, {
  hour12: false,
}, "hour12 option and hc Unicode extension value are present.");

assertLocale(defaultLocaleWithHourCycle, defaultLocale, {
  hourCycle: "h23",
}, "hourCycle option and hc Unicode extension value are present.");

assertLocale(defaultLocaleWithHourCycle, defaultLocaleWithHourCycle, {
}, "Only hc Unicode extension value is present.");

// And make sure the hc Unicode extension doesn't get added if it's not present
// in the requested locale.
assertLocale(defaultLocale, defaultLocale, {
  hour12: false,
  hourCycle: "h23",
}, "hour12 and hourCycle options are present, but no hc Unicode extension value.");

assertLocale(defaultLocale, defaultLocale, {
  hour12: false,
}, "hourCycle option is present, but no hc Unicode extension value.");

assertLocale(defaultLocale, defaultLocale, {
  hourCycle: "h23",
}, "hourCycle option is present, but no hc Unicode extension value.");

assertLocale(defaultLocale, defaultLocale, {
}, "No options are present and no hc Unicode extension value.");
