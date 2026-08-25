// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaintime
description: Hour argument defaults to 0 if not given
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const explicit = new Temporal.PlainTime(undefined);
TemporalHelpers.assertPlainTime(explicit, 0, 0, 0, 0, 0, 0, "explicit");

const implicit = new Temporal.PlainTime();
TemporalHelpers.assertPlainTime(implicit, 0, 0, 0, 0, 0, 0, "implicit");
