// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/

// Ensure we properly quote strings which can contain the NUL character before
// returning them to the user to avoid cutting off any trailing characters.

assert.throws(TypeError, () => "foo\0bar" in "asdf\0qwertz", "bar");
assert.throws(TypeError, () => "foo\0bar" in "asdf\0qwertz", "qwertz");

if (this.Intl) {
    assert.throws(RangeError, () => Intl.getCanonicalLocales("de\0Latn"), "Latn");

    assert.throws(RangeError, () => Intl.Collator.supportedLocalesOf([], {localeMatcher:"lookup\0cookie"}), "cookie");
    assert.throws(RangeError, () => new Intl.Collator("en", {localeMatcher: "lookup\0cookie"}), "cookie");
    assert.throws(RangeError, () => new Intl.Collator("en", {usage: "sort\0cookie"}), "cookie");
    assert.throws(RangeError, () => new Intl.Collator("en", {caseFirst: "upper\0cookie"}), "cookie");
    assert.throws(RangeError, () => new Intl.Collator("en", {sensitivity: "base\0cookie"}), "cookie");

    assert.throws(RangeError, () => Intl.DateTimeFormat.supportedLocalesOf([], {localeMatcher:"lookup\0cookie"}), "cookie");
    assert.throws(RangeError, () => new Intl.DateTimeFormat("en", {localeMatcher: "lookup\0cookie"}), "cookie");
    assert.throws(RangeError, () => new Intl.DateTimeFormat("en", {hourCycle: "h24\0cookie"}), "cookie");
    assert.throws(RangeError, () => new Intl.DateTimeFormat("en", {weekday: "narrow\0cookie"}), "cookie");
    assert.throws(RangeError, () => new Intl.DateTimeFormat("en", {era: "narrow\0cookie"}), "cookie");
    assert.throws(RangeError, () => new Intl.DateTimeFormat("en", {year: "2-digit\0cookie"}), "cookie");
    assert.throws(RangeError, () => new Intl.DateTimeFormat("en", {month: "2-digit\0cookie"}), "cookie");
    assert.throws(RangeError, () => new Intl.DateTimeFormat("en", {day: "2-digit\0cookie"}), "cookie");
    assert.throws(RangeError, () => new Intl.DateTimeFormat("en", {hour: "2-digit\0cookie"}), "cookie");
    assert.throws(RangeError, () => new Intl.DateTimeFormat("en", {minute: "2-digit\0cookie"}), "cookie");
    assert.throws(RangeError, () => new Intl.DateTimeFormat("en", {second: "2-digit\0cookie"}), "cookie");
    assert.throws(RangeError, () => new Intl.DateTimeFormat("en", {formatMatcher: "basic\0cookie"}), "cookie");
    assert.throws(RangeError, () => new Intl.DateTimeFormat("en", {timeZone: "UTC\0cookie"}), "cookie");

    assert.throws(RangeError, () => Intl.NumberFormat.supportedLocalesOf([], {localeMatcher:"lookup\0cookie"}), "cookie");
    assert.throws(RangeError, () => new Intl.NumberFormat("en", {localeMatcher: "lookup\0cookie"}), "cookie");
    assert.throws(RangeError, () => new Intl.NumberFormat("en", {style: "decimal\0cookie"}), "cookie");
    assert.throws(RangeError, () => new Intl.NumberFormat("en", {currency: "USD\0cookie"}), "cookie");
    assert.throws(RangeError, () => new Intl.NumberFormat("en", {currencyDisplay: "code\0cookie"}), "cookie");

    assert.throws(RangeError, () => Intl.PluralRules.supportedLocalesOf([], {localeMatcher:"lookup\0cookie"}), "cookie");
    assert.throws(RangeError, () => new Intl.PluralRules("en", {localeMatcher: "lookup\0cookie"}), "cookie");
    assert.throws(RangeError, () => new Intl.PluralRules("en", {type: "cardinal\0cookie"}), "cookie");

    assert.throws(RangeError, () => Intl.RelativeTimeFormat.supportedLocalesOf([], {localeMatcher:"lookup\0cookie"}), "cookie");
    assert.throws(RangeError, () => new Intl.RelativeTimeFormat("en", {localeMatcher: "lookup\0cookie"}), "cookie");
    assert.throws(RangeError, () => new Intl.RelativeTimeFormat("en", {style: "long\0cookie"}), "cookie");
    assert.throws(RangeError, () => new Intl.RelativeTimeFormat("en", {numeric: "auto\0cookie"}), "cookie");
    assert.throws(RangeError, () => new Intl.RelativeTimeFormat().format(1, "day\0cookie"), "cookie");

    assert.throws(RangeError, () => new Intl.Locale("de\0keks"), "keks");
    assert.throws(RangeError, () => new Intl.Locale("de", {language: "it\0biscotto"}), "biscotto");
    assert.throws(RangeError, () => new Intl.Locale("th", {script: "Thai\0คุกกี้"}), "\\u0E04\\u0E38\\u0E01\\u0E01\\u0E35\\u0E49");
    assert.throws(RangeError, () => new Intl.Locale("en", {region: "GB\0biscuit"}), "biscuit");

    assert.throws(RangeError, () => new Intl.Locale("und", {calendar: "gregory\0F1"}), "F1");
    assert.throws(RangeError, () => new Intl.Locale("und", {collation: "phonebk\0F2"}), "F2");
    assert.throws(RangeError, () => new Intl.Locale("und", {hourCycle: "h24\0F3"}), "F3");
    assert.throws(RangeError, () => new Intl.Locale("und", {caseFirst: "false\0F4"}), "F4");
    assert.throws(RangeError, () => new Intl.Locale("und", {numberingSystem: "arab\0F5"}), "F5");
}
