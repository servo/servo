// Copyright 2025 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.datetimeformat.prototype.resolvedoptions
description: >
  Resolved hour-cycle when using Unicode extension values and options.
locale: [en]
---*/

var tests = [
  // Unicode extension value is present and supported. Different options value
  // present and supported. Unicode extension value is ignored and not reflected
  // in the resolved locale.
  {
    locale: "en-u-hc-h23",
    hourCycle: "h11",
    resolved: {
      locale: "en",
      hourCycle: "h11",
    }
  },

  // Unicode extension value is present and supported. Options value present and
  // supported. Unicode extension value is equal to options value. Unicode
  // extension value is reflected in the resolved locale.
  {
    locale: "en-u-hc-h23",
    hourCycle: "h23",
    resolved: {
      locale: "en-u-hc-h23",
      hourCycle: "h23",
    }
  },
];

for (var {locale, hourCycle, resolved} of tests) {
  var dtf = new Intl.DateTimeFormat(locale, {hour: "numeric", hourCycle});
  var resolvedOptions = dtf.resolvedOptions();

  assert.sameValue(
    resolvedOptions.locale,
    resolved.locale,
    `Resolved locale for locale=${locale} with hourCycle=${hourCycle}`
  );
  assert.sameValue(
    resolvedOptions.hourCycle,
    resolved.hourCycle,
    `Resolved hourCycle for locale=${locale} with hourCycle=${hourCycle}`
  );
}
