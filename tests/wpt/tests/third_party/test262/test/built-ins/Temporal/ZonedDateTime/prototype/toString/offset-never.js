// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.tostring
description: Omits offset if offset = never.
features: [Temporal]
---*/

const zdt1 = Temporal.ZonedDateTime.from("1976-11-18T15:23+00:00[UTC]");

assert.sameValue(zdt1.toString({ offset: "never" }), "1976-11-18T15:23:00[UTC]");
