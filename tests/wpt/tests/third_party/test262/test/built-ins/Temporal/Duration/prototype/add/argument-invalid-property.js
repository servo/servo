// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.add
description: temporalDurationLike object must contain at least one correctly spelled property
features: [Temporal]
---*/

const instance = new Temporal.Duration(0, 0, 0, 1, 2, 3, 4, 987, 654, 321);

assert.throws(
  TypeError,
  () => instance.add({}),
  "Throws TypeError if no property is present"
);

assert.throws(
  TypeError,
  () => instance.add({ nonsense: true }),
  "Throws TypeError if no recognized property is present"
);

assert.throws(
  TypeError,
  () => instance.add({ sign: 1 }),
  "Sign property is not recognized"
);
