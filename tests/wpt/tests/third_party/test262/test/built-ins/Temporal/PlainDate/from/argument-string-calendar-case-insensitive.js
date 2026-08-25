// Copyright (C) 2026 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.from
description: The calendar name is case-insensitive
features: [Temporal]
---*/

const arg = "1976-11-18T00:00[u-ca=ISO8601]";
const result = Temporal.PlainDate.from(arg);
assert.sameValue(result.calendarId, "iso8601", "Calendar is case-insensitive");
