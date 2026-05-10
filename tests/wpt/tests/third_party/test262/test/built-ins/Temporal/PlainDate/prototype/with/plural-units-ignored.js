// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.with
description: Plural units are not valid in the property bag
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const plainDate = new Temporal.PlainDate(1976, 11, 18);

const withPlural = plainDate.with({ months: 12, day: 15 });
TemporalHelpers.assertPlainDate(withPlural, 1976, 11, "M11", 15, "Plural units in the property bag should ignored");
