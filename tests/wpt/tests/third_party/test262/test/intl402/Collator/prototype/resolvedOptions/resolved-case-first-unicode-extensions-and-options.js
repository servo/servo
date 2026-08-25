// Copyright 2025 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.collator.prototype.resolvedoptions
description: >
  Resolved caseFirst when using Unicode extension values and options.
locale: [en]
---*/

var tests = [
  // Unicode extension value is present and supported. Different options value
  // present and supported. Unicode extension value is ignored and not reflected
  // in the resolved locale.
  {
    locale: "en-u-kf-lower",
    caseFirst: "upper",
    resolved: {
      locale: "en",
      caseFirst: "upper",
    }
  },

  // Unicode extension value is present and supported. Options value present and
  // supported. Unicode extension value is equal to options value. Unicode
  // extension value is reflected in the resolved locale.
  {
    locale: "en-u-kf-lower",
    caseFirst: "lower",
    resolved: {
      locale: "en-u-kf-lower",
      caseFirst: "lower",
    }
  },
];

for (var {locale, caseFirst, resolved} of tests) {
  var coll = new Intl.Collator(locale, {caseFirst});
  var resolvedOptions = coll.resolvedOptions();

  // Skip if this implementation doesn't support the optional "kf" extension key.
  if (!resolvedOptions.hasOwnProperty("caseFirst")) {
    continue;
  }

  assert.sameValue(
    resolvedOptions.locale,
    resolved.locale,
    `Resolved locale for locale=${locale} with caseFirst=${caseFirst}`
  );
  assert.sameValue(
    resolvedOptions.caseFirst,
    resolved.caseFirst,
    `Resolved numeric for locale=${locale} with caseFirst=${caseFirst}`
  );
}
