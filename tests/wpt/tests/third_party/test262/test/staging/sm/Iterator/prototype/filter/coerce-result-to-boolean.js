// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
features:
  - IsHTMLDDA
  - iterator-helpers
info: |
  Iterator is not enabled unconditionally
description: |
  pending
esid: pending
---*/

// All truthy values are kept.
const truthyValues = [true, 1, [], {}, 'test'];
for (const value of [...truthyValues].values().filter(x => x)) {
  assert.sameValue(truthyValues.shift(), value);
}

// All falsy values are filtered out.
const falsyValues = [false, 0, '', null, undefined, NaN, -0, 0n, $262.IsHTMLDDA];
const result = falsyValues.values().filter(x => x).next();
assert.sameValue(result.done, true);
assert.sameValue(result.value, undefined);

