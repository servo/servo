// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
flags:
  - noStrict
description: |
  Dense array properties shouldn't be modified when they're frozen
info: bugzilla.mozilla.org/show_bug.cgi?id=1310744
esid: pending
---*/
/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 * Author: Emilio Cobos Álvarez <ecoal95@gmail.com>
 */

var a = Object.freeze([4, 5, 1]);

function assertArrayIsExpected() {
  assert.sameValue(a.length, 3);
  assert.sameValue(a[0], 4);
  assert.sameValue(a[1], 5);
  assert.sameValue(a[2], 1);
}

assert.throws(TypeError, () => a.reverse());
assert.throws(TypeError, () => a.shift());
assert.throws(TypeError, () => a.unshift(0));
assert.throws(TypeError, () => a.sort(function() {}));
assert.throws(TypeError, () => a.pop());
assert.throws(TypeError, () => a.fill(0));
assert.throws(TypeError, () => a.splice(0, 1, 1));
assert.throws(TypeError, () => a.push("foo"));
assert.throws(TypeError, () => { "use strict"; a.length = 5; });
assert.throws(TypeError, () => { "use strict"; a[2] = "foo"; });
assert.throws(TypeError, () => { "use strict"; delete a[0]; });
assert.throws(TypeError, () => a.splice(Math.a));

// Shouldn't throw, since this is not strict mode, but shouldn't change the
// value of the property.
a.length = 5;
a[2] = "foo";
assert.sameValue(delete a[0], false);

assertArrayIsExpected();
