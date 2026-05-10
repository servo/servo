// Copyright 2025 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.collator.prototype.resolvedoptions
description: >
  Resolved collation when using Unicode extension values and options.
locale: [en, de]
---*/

var tests = [
  // Unicode extension value is present and supported. Options value present,
  // but unsupported. Unicode extension value is used and reflected in the
  // resolved locale.
  {
    locale: "de-u-co-phonebk",
    collation: "pinyin",
    resolved: {
      locale: "de-u-co-phonebk",
      collation: "phonebk",
    },
  },

  // Unicode extension value is present, but unsupported. Options value present,
  // but also unsupported. Default collation is used.
  {
    locale: "en-u-co-phonebk",
    collation: "pinyin",
    resolved: {
      locale: "en",
      collation: "default",
    },
  },

  // Unicode extension value is present and supported. Different options value
  // present and supported. Unicode extension value is ignored and not reflected
  // in the resolved locale.
  {
    locale: "de-u-co-phonebk",
    collation: "eor",
    resolved: {
      locale: "de",
      collation: "eor",
    }
  },

  // Unicode extension value is present and supported. Options value present and
  // supported. Unicode extension value is equal to options value. Unicode
  // extension value is reflected in the resolved locale.
  {
    locale: "de-u-co-phonebk",
    collation: "phonebk",
    resolved: {
      locale: "de-u-co-phonebk",
      collation: "phonebk",
    }
  },
];

for (var {locale, collation, resolved} of tests) {
  var coll = new Intl.Collator(locale, {collation});
  var resolvedOptions = coll.resolvedOptions();

  assert.sameValue(
    resolvedOptions.locale,
    resolved.locale,
    `Resolved locale for locale=${locale} with collation=${collation}`
  );
  assert.sameValue(
    resolvedOptions.collation,
    resolved.collation,
    `Resolved collation for locale=${locale} with collation=${collation}`
  );
}
