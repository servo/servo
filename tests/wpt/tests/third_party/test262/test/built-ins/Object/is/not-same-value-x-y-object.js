// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 19.1.2.10
description: >
    Object.is ( value1, value2 )

    ...
    10. Return true if x and y are the same Object value. Otherwise, return false.
---*/

assert.sameValue(Object.is({}, {}), false, "`Object.is({}, {})` returns `false`");
assert.sameValue(
  Object.is(Object(), Object()),
  false,
  "`Object.is(Object(), Object())` returns `false`"
);
assert.sameValue(
  Object.is(new Object(), new Object()),
  false,
  "`Object.is(new Object(), new Object())` returns `false`"
);
assert.sameValue(
  Object.is(Object(0), Object(0)),
  false,
  "`Object.is(Object(0), Object(0))` returns `false`"
);
assert.sameValue(
  Object.is(new Object(''), new Object('')),
  false,
  "`Object.is(new Object(''), new Object(''))` returns `false`"
);
