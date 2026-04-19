// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
includes: [compareArray.js]
description: |
  pending
esid: pending
---*/

/* Resetting of inner capture groups across quantified capturing parens. */
var re = /(?:(f)(o)(o)|(b)(a)(r))*/;
var str = 'foobar';
var actual = re.exec(str);

assert.compareArray(actual, ['foobar', undefined, undefined, undefined, 'b', 'a', 'r']);
assert.sameValue(actual.index, 0);
assert.sameValue(actual.input, str);
