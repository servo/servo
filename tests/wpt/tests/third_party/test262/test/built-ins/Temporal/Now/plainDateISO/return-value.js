// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.now.plaindateiso
description: Functions when time zone argument is omitted
features: [Temporal]
---*/

const d = Temporal.Now.plainDateISO();
assert(d instanceof Temporal.PlainDate);
assert.sameValue(d.calendarId, "iso8601", "calendar string should be iso8601");
