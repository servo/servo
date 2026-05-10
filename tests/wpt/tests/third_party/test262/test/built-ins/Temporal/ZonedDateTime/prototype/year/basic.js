// Copyright (C) 2026 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-get-temporal.zoneddatetime.prototype.year
description: The "year" property of Temporal.ZonedDateTime.prototype
features: [Temporal]
---*/

assert.sameValue((new Temporal.ZonedDateTime(0n, "UTC")).year, 1970);
assert.sameValue(Temporal.ZonedDateTime.from('2019-03-15T15:30:26+00:00[UTC]').year, 2019);
