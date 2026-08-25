// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Temporal.PlainDate.prototype.subtract throws a RangeError if the calculation crosses a limit
esid: sec-temporal.plaindate.prototype.subtract
features: [Temporal]
---*/

const min = Temporal.PlainDate.from("-271821-04-19");
const max = Temporal.PlainDate.from("+275760-09-13");
["reject", "constrain"].forEach((overflow) => {
  assert.throws(RangeError, () => min.subtract({ days: 1 }, { overflow }), `min with ${overflow}`);
  assert.throws(RangeError, () => max.subtract({ days: -1 }, { overflow }), `max with ${overflow}`);
});
