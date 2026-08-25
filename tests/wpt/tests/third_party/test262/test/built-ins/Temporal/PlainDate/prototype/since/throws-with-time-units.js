// Copyright (C) 2026 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.since
description: Throws if largestUnit or smallestUnit is a time unit
features: [Temporal]
---*/

const from = new Temporal.PlainDate(2021, 7, 16);
const to = new Temporal.PlainDate(2021, 7, 17);

const units = ['hour', 'minute', 'second', 'millisecond', 'microsecond', 'nanosecond'];

for (const largestUnit of units) {
  for (const smallestUnit of units) {
    assert.throws(RangeError, () => from.since(to, { largestUnit, smallestUnit }),
                 `Can't use ${largestUnit} and ${smallestUnit} as largestUnit and smallestUnit for PlainDate`);
  }
}
