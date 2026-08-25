// Copyright 2025 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.collator.prototype.resolvedoptions
description: >
  Resolved numeric when using Unicode extension values and options.
locale: [en]
---*/

var tests = [
  // Unicode extension value is present and supported. Different options value
  // present and supported. Unicode extension value is ignored and not reflected
  // in the resolved locale.
  {
    locale: "en-u-kn-false",
    numeric: true,
    resolved: {
      locale: "en",
      numeric: true,
    }
  },

  // Unicode extension value is present and supported. Options value present and
  // supported. Unicode extension value is equal to options value. Unicode
  // extension value is reflected in the resolved locale.
  {
    locale: "en-u-kn-true",
    numeric: true,
    resolved: {
      locale: "en-u-kn",
      numeric: true,
    }
  },
];

for (var {locale, numeric, resolved} of tests) {
  var coll = new Intl.Collator(locale, {numeric});
  var resolvedOptions = coll.resolvedOptions();

  // Skip if this implementation doesn't support the optional "kn" extension key.
  if (!resolvedOptions.hasOwnProperty("numeric")) {
    continue;
  }

  assert.sameValue(
    resolvedOptions.locale,
    resolved.locale,
    `Resolved locale for locale=${locale} with numeric=${numeric}`
  );
  assert.sameValue(
    resolvedOptions.numeric,
    resolved.numeric,
    `Resolved numeric for locale=${locale} with numeric=${numeric}`
  );
}
