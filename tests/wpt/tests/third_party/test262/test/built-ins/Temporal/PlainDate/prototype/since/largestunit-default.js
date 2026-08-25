// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.since
description: Default value for largestUnit option is days
includes: [compareArray.js, temporalHelpers.js]
features: [Temporal]
---*/

const feb20 = Temporal.PlainDate.from("2020-02-01");
const feb21 = Temporal.PlainDate.from("2021-02-01");
TemporalHelpers.assertDuration(feb21.since(feb20), 0, 0, 0, /* days = */ 366, 0, 0, 0, 0, 0, 0, "no options");
TemporalHelpers.assertDuration(feb21.since(feb20, undefined), 0, 0, 0, /* days = */ 366, 0, 0, 0, 0, 0, 0, "undefined options");
TemporalHelpers.assertDuration(feb21.since(feb20, {}), 0, 0, 0, /* days = */ 366, 0, 0, 0, 0, 0, 0, "no largestUnit");
TemporalHelpers.assertDuration(feb21.since(feb20, { largestUnit: undefined }), 0, 0, 0, /* days = */ 366, 0, 0, 0, 0, 0, 0, "undefined largestUnit");
TemporalHelpers.assertDuration(feb21.since(feb20, { largestUnit: "days" }), 0, 0, 0, /* days = */ 366, 0, 0, 0, 0, 0, 0, "days");
TemporalHelpers.assertDuration(feb21.since(feb20, { largestUnit: "auto" }), 0, 0, 0, /* days = */ 366, 0, 0, 0, 0, 0, 0, "auto");
TemporalHelpers.assertDuration(feb21.since(feb20, () => {}), 0, 0, 0, /* days = */ 366, 0, 0, 0, 0, 0, 0, "no largestUnit (function)");
