// Copyright (C) 2018 Bloomberg LP. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.instant.prototype.equals
description: Temporal.Instant.equals works cross-epoch
features: [Temporal]
---*/

const i1 = Temporal.Instant.from("1963-02-13T09:36:29.123456789Z");
const i2 = Temporal.Instant.from("1976-11-18T15:23:30.123456789Z");
const i3 = Temporal.Instant.from("1981-12-15T14:34:31.987654321Z");

// pre epoch equal
assert(i1.equals(i1))

// epoch equal
assert(i2.equals(i2))

// cross epoch unequal
assert(!i1.equals(i2))

// epoch unequal
assert(!i2.equals(i3))
