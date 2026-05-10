// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 19.1.2.10
description: >
    Object.is ( value1, value2 )

    7.2.9 SameValue(x, y)

    ...
    8. If Type(x) is Boolean, then
      a. If x and y are both true or both false,
          return true; otherwise, return false.
---*/

assert.sameValue(Object.is(true, false), false, "`Object.is(true, false)` returns `false`");
assert.sameValue(Object.is(false, true), false, "`Object.is(false, true)` returns `false`");
assert.sameValue(Object.is(true, 1), false, "`Object.is(true, 1)` returns `false`");
assert.sameValue(Object.is(false, 0), false, "`Object.is(false, 0)` returns `false`");
assert.sameValue(Object.is(true, {}), false, "`Object.is(true, {})` returns `false`");
assert.sameValue(Object.is(true, undefined), false, "`Object.is(true, undefined)` returns `false`");
assert.sameValue(Object.is(false, undefined), false, "`Object.is(false, undefined)` returns `false`");
assert.sameValue(Object.is(true, null), false, "`Object.is(true, null)` returns `false`");
assert.sameValue(Object.is(false, null), false, "`Object.is(false, null)` returns `false`");
assert.sameValue(Object.is(true, NaN), false, "`Object.is(true, NaN)` returns `false`");
assert.sameValue(Object.is(false, NaN), false, "`Object.is(false, NaN)` returns `false`");
assert.sameValue(Object.is(true, ''), false, "`Object.is(true, '')` returns `false`");
assert.sameValue(Object.is(false, ''), false, "`Object.is(false, '')` returns `false`");
assert.sameValue(Object.is(true, []), false, "`Object.is(true, [])` returns `false`");
assert.sameValue(Object.is(false, []), false, "`Object.is(false, [])` returns `false`");
