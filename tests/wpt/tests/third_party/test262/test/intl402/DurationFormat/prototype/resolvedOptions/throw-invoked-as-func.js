// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.DurationFormat.prototype.resolvedOptions
description: basic tests internal slot initialization and call receiver errors
info: |
  Intl.DurationFormat.prototype.resolvedOptions ( )
  (...)
    2. Perform ? RequireInternalSlot(df, [[InitializedDurationFormat]]).
features: [Intl.DurationFormat]
---*/

const df = new Intl.DurationFormat();

// Perform ? RequireInternalSlot(df, [[InitializedDurationFormat]]).
let f = df['resolvedOptions'];

assert.sameValue(typeof f, 'function');
assert.throws(TypeError, () => { f() });

