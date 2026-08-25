// Copyright (C) 2026 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.instant.prototype.tolocalestring
description: Formatting a Plain* object should not break the formatter for Instant.
features: [Temporal]
---*/

const datestring = '2026-03-29T02:30:15+01:00';
const instant = Temporal.Instant.from(datestring);
const pdt = Temporal.PlainDateTime.from(datestring);

const legacyDate = new Date(instant.epochMilliseconds);
const expected1 = legacyDate.toLocaleString("en", { timeStyle: "long", timeZone: "America/Los_Angeles" });
const expected2 = legacyDate.toLocaleString("en", { timeStyle: "long", timeZone: "Europe/Berlin" });

pdt.toLocaleString("en", { timeStyle: "long" });

assert.sameValue(
  instant.toLocaleString('en', { timeStyle: 'long', timeZone: 'America/Los_Angeles' }),
  expected1
);

assert.sameValue(
  instant.toLocaleString('en', { timeStyle: 'long', timeZone: 'Europe/Berlin' }),
  expected2
);

new Temporal.PlainDate(2011, 12, 30).toLocaleString("en");

assert.sameValue(
  new Temporal.Instant(0n).toLocaleString("en", { era: "narrow" }),
  new Date(0).toLocaleString(["en"], { era: "narrow" })
);
