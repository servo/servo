// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.datetimeformat
description: >
  Tests that fallbacks for deprecated calendars are selected from one of the
  values returned from `AvailableCalendars`.
info: |
  CreateDateTimeFormat ( _newTarget_, _locales_, _options_, _required_, _defaults_ )

  ...
  9. If _resolvedCalendar_ is *"islamic"* or *"islamic-rgsa"*, then
    a. Let _fallbackCalendar_ be an implementation- and locale-defined calendar type that is one of the values returned from AvailableCalendars.
    b. Set _resolvedCalendar_ to CanonicalizeUValue(*"ca"*, _fallbackCalendar_).
    c. If the ECMAScript implementation has a mechanism for reporting diagnostic warning messages, a warning should be issued.
  10. Set _dateTimeFormat_.[[Calendar]] to _resolvedCalendar_.
locale: [en]
features: [Intl.Era-monthcode]
---*/

const availableCalendars = [
  "buddhist",
	"chinese",
	"coptic",
	"dangi",
	"ethioaa",
	"ethiopic",
	"gregory",
	"hebrew",
	"indian",
	"islamic-civil",
	"islamic-tbla",
	"islamic-umalqura",
	"iso8601",
	"japanese",
	"persian",
	"roc",
];

const islamic = new Intl.DateTimeFormat("en", { calendar: "islamic" });
assert.sameValue(availableCalendars.includes(islamic.resolvedOptions().calendar), true, "no valid fallback for 'islamic' calendar option");

const islamicRgsa  = new Intl.DateTimeFormat("en", { calendar: "islamic-rgsa" });
assert.sameValue(availableCalendars.includes(islamicRgsa.resolvedOptions().calendar), true, "no valid fallback for 'islamic-rgsa' calendar option");

const islamicUExtension = new Intl.DateTimeFormat("en-u-ca-islamic");
assert.sameValue(availableCalendars.includes(islamicUExtension.resolvedOptions().calendar), true, "no valid fallback for 'islamic' calendar u extension");

const islamicRgsaUExtension = new Intl.DateTimeFormat("en-u-ca-islamic-rgsa");
assert.sameValue(availableCalendars.includes(islamicRgsaUExtension.resolvedOptions().calendar), true, "no valid fallback for 'islamic-rgsa' calendar u extension");
