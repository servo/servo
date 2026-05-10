// Copyright 2025 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.DurationFormat.prototype.resolvedOptions
description: >
  Resolved numbering system when using Unicode extension values and options.
locale: [en]
---*/

var tests = [
  // Unicode extension value is present and supported. Options value present,
  // but unsupported. Unicode extension value is used and reflected in the
  // resolved locale.
  {
    locale: "en-u-nu-arab",
    numberingSystem: "invalid",
    resolved: {
      locale: "en-u-nu-arab",
      numberingSystem: "arab",
    },
  },

  // Unicode extension value is present, but unsupported. Options value present,
  // but also unsupported. Default numbering system is used.
  {
    locale: "en-u-nu-invalid",
    numberingSystem: "invalid2",
    resolved: {
      locale: "en",
      numberingSystem: "latn",
    },
  },

  // Unicode extension value is present and supported. Different options value
  // present and supported. Unicode extension value is ignored and not reflected
  // in the resolved locale.
  {
    locale: "en-u-nu-latn",
    numberingSystem: "arab",
    resolved: {
      locale: "en",
      numberingSystem: "arab",
    }
  },

  // Unicode extension value is present and supported. Options value present and
  // supported. Unicode extension value is equal to options value. Unicode
  // extension value is reflected in the resolved locale.
  {
    locale: "en-u-nu-arab",
    numberingSystem: "arab",
    resolved: {
      locale: "en-u-nu-arab",
      numberingSystem: "arab",
    }
  },
];

for (var {locale, numberingSystem, resolved} of tests) {
  var df = new Intl.DurationFormat(locale, {numberingSystem});
  var resolvedOptions = df.resolvedOptions();

  assert.sameValue(
    resolvedOptions.locale,
    resolved.locale,
    `Resolved locale for locale=${locale} with numberingSystem=${numberingSystem}`
  );
  assert.sameValue(
    resolvedOptions.numberingSystem,
    resolved.numberingSystem,
    `Resolved numberingSystem for locale=${locale} with numberingSystem=${numberingSystem}`
  );
}
