// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal-objects
description: Temporal has own property names
features: [Temporal]
---*/

const keys = Object.getOwnPropertyNames(Temporal);

assert(keys.indexOf("Instant") > -1, "Instant");
assert(keys.indexOf("PlainDate") > -1, "PlainDate");
assert(keys.indexOf("PlainTime") > -1, "PlainTime");
assert(keys.indexOf("PlainDateTime") > -1, "PlainDateTime");
assert(keys.indexOf("ZonedDateTime") > -1, "ZonedDateTime");
assert(keys.indexOf("PlainYearMonth") > -1, "PlainYearMonth");
assert(keys.indexOf("PlainMonthDay") > -1, "PlainMonthDay");
assert(keys.indexOf("Duration") > -1, "Duration");
assert(keys.indexOf("Now") > -1, "Now");
