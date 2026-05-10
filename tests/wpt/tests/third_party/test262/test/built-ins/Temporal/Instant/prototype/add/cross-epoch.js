// Copyright (C) 2018 Bloomberg LP. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal-instant.prototype.add
description: Cross-epoch arithmetic.
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const inst = Temporal.Instant.from("1969-12-25T12:23:45.678901234Z");

// cross epoch in ms
const one = inst.subtract({
  hours: 240,
  nanoseconds: 800
});
const two = inst.add({
  hours: 240,
  nanoseconds: 800
});
const three = two.subtract({
  hours: 480,
  nanoseconds: 1600
});
const four = one.add({
  hours: 480,
  nanoseconds: 1600
});
TemporalHelpers.assertInstantsEqual(one, Temporal.Instant.from("1969-12-15T12:23:45.678900434Z"),
                    `(${ inst }).subtract({ hours: 240, nanoseconds: 800 }) = ${ one }`);
TemporalHelpers.assertInstantsEqual(two, Temporal.Instant.from("1970-01-04T12:23:45.678902034Z"),
                    `(${ inst }).add({ hours: 240, nanoseconds: 800 }) = ${ two }`);
TemporalHelpers.assertInstantsEqual(three, one, `(${ two }).subtract({ hours: 480, nanoseconds: 1600 }) = ${ one }`);
TemporalHelpers.assertInstantsEqual(four, two, `(${ one }).add({ hours: 480, nanoseconds: 1600 }) = ${ two }`);
