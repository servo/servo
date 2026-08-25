// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.toplaindatetime
description: The time is assumed to be midnight if not given
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const date = new Temporal.PlainDate(2000, 5, 2);

const explicit = date.toPlainDateTime(undefined);
TemporalHelpers.assertPlainDateTime(explicit, 2000, 5, "M05", 2, 0, 0, 0, 0, 0, 0, "default time is midnight - explicit");

const implicit = date.toPlainDateTime();
TemporalHelpers.assertPlainDateTime(implicit, 2000, 5, "M05", 2, 0, 0, 0, 0, 0, 0, "default time is midnight - implicit");
