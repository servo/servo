// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 19.1.2.10
description: >
    Object.is ( value1, value2 )

    7.2.9 SameValue(x, y)

    ...
    3. If Type(x) is different from Type(y), return false.
    ...

---*/

var a = {};

assert.sameValue(Object.is(a, true), false, "`Object.is(a, true)` returns `false`");
assert.sameValue(Object.is(a, ''), false, "`Object.is(a, '')` returns `false`");
assert.sameValue(Object.is(a, 0), false, "`Object.is(a, 0)` returns `false`");
assert.sameValue(
  Object.is(a, undefined),
  false,
  "`Object.is(a, undefined)` returns `false`"
);

assert.sameValue(Object.is(NaN, true), false, "`Object.is(NaN, true)` returns `false`");
assert.sameValue(Object.is(NaN, ''), false, "`Object.is(NaN, '')` returns `false`");
assert.sameValue(Object.is(NaN, a), false, "`Object.is(NaN, a)` returns `false`");
assert.sameValue(
  Object.is(NaN, undefined),
  false,
  "`Object.is(NaN, undefined)` returns `false`"
);
assert.sameValue(Object.is(NaN, null), false, "`Object.is(NaN, null)` returns `false`");

assert.sameValue(Object.is(true, 0), false, "`Object.is(true, 0)` returns `false`");
assert.sameValue(Object.is(true, a), false, "`Object.is(true, a)` returns `false`");
assert.sameValue(
  Object.is(true, undefined),
  false,
  "`Object.is(true, undefined)` returns `false`"
);
assert.sameValue(Object.is(true, null), false, "`Object.is(true, null)` returns `false`");
assert.sameValue(Object.is(true, NaN), false, "`Object.is(true, NaN)` returns `false`");
assert.sameValue(Object.is(true, ''), false, "`Object.is(true, '')` returns `false`");

assert.sameValue(Object.is(false, 0), false, "`Object.is(false, 0)` returns `false`");
assert.sameValue(Object.is(false, a), false, "`Object.is(false, a)` returns `false`");
assert.sameValue(
  Object.is(false, undefined),
  false,
  "`Object.is(false, undefined)` returns `false`"
);
assert.sameValue(Object.is(false, null), false, "`Object.is(false, null)` returns `false`");
assert.sameValue(Object.is(false, NaN), false, "`Object.is(false, NaN)` returns `false`");
assert.sameValue(Object.is(false, ''), false, "`Object.is(false, '')` returns `false`");

assert.sameValue(Object.is(0, true), false, "`Object.is(0, true)` returns `false`");
assert.sameValue(Object.is(0, a), false, "`Object.is(0, a)` returns `false`");
assert.sameValue(
  Object.is(0, undefined),
  false,
  "`Object.is(0, undefined)` returns `false`"
);
assert.sameValue(Object.is(0, null), false, "`Object.is(0, null)` returns `false`");
assert.sameValue(Object.is(0, NaN), false, "`Object.is(0, NaN)` returns `false`");
assert.sameValue(Object.is(0, ''), false, "`Object.is(0, '')` returns `false`");
