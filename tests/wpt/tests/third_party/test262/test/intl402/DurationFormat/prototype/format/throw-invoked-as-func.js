// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.DurationFormat.prototype.format
description: basic tests internal slot initialization and call receiver errors
info: |
  Intl.DurationFormat.prototype.format ( duration )
  (...)
    2. Perform ? RequireInternalSlot(df, [[InitializedDurationFormat]]).
features: [Intl.DurationFormat]
---*/

const df = new Intl.DurationFormat();

let f = df["format"];

assert.sameValue(typeof f, "function");
assert.throws(TypeError, () => {
  f({ hours: 1, minutes: 46, seconds: 40 });
});
