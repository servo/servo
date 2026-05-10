// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaintime
description: Minute argument defaults to 0 if not given
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const hour = 12;

const explicit = new Temporal.PlainTime(hour, undefined);
TemporalHelpers.assertPlainTime(explicit, hour, 0, 0, 0, 0, 0, "explicit");

const implicit = new Temporal.PlainTime(hour);
TemporalHelpers.assertPlainTime(implicit, hour, 0, 0, 0, 0, 0, "implicit");
