// Copyright 2025 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.datetimeformat.prototype.resolvedoptions
description: >
  Resolved calendar when using Unicode extension values and options.
locale: [en]
---*/

var tests = [
  // Unicode extension value is present and supported. Options value present,
  // but unsupported. Unicode extension value is used and reflected in the
  // resolved locale.
  {
    locale: "en-u-ca-iso8601",
    calendar: "invalid",
    resolved: {
      locale: "en-u-ca-iso8601",
      calendar: "iso8601",
    },
  },

  // Unicode extension value is present, but unsupported. Options value present,
  // but also unsupported. Default calendar is used.
  {
    locale: "en-u-ca-invalid",
    calendar: "invalid2",
    resolved: {
      locale: "en",
      calendar: "gregory",
    },
  },

  // Unicode extension value is present and supported. Different options value
  // present and supported. Unicode extension value is ignored and not reflected
  // in the resolved locale.
  {
    locale: "en-u-ca-gregory",
    calendar: "iso8601",
    resolved: {
      locale: "en",
      calendar: "iso8601",
    }
  },

  // Unicode extension value is present and supported. Options value present and
  // supported. Unicode extension value is equal to options value. Unicode
  // extension value is reflected in the resolved locale.
  {
    locale: "en-u-ca-iso8601",
    calendar: "iso8601",
    resolved: {
      locale: "en-u-ca-iso8601",
      calendar: "iso8601",
    }
  },
];

for (var {locale, calendar, resolved} of tests) {
  var dtf = new Intl.DateTimeFormat(locale, {calendar});
  var resolvedOptions = dtf.resolvedOptions();

  assert.sameValue(
    resolvedOptions.locale,
    resolved.locale,
    `Resolved locale for locale=${locale} with calendar=${calendar}`
  );
  assert.sameValue(
    resolvedOptions.calendar,
    resolved.calendar,
    `Resolved calendar for locale=${locale} with calendar=${calendar}`
  );
}
