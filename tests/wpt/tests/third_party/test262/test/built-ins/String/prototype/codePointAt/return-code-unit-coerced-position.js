// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 21.1.3.3
description: >
  Return value on coerced values on ToInteger(position).
info: |
  21.1.3.3 String.prototype.codePointAt ( pos )

  ...
  4. Let position be ToInteger(pos).
  ...

---*/

assert.sameValue('\uD800\uDC00'.codePointAt(''), 65536);
assert.sameValue('\uD800\uDC00'.codePointAt('0'), 65536);
assert.sameValue('\uD800\uDC00'.codePointAt(NaN), 65536);
assert.sameValue('\uD800\uDC00'.codePointAt(false), 65536);
assert.sameValue('\uD800\uDC00'.codePointAt(null), 65536);
assert.sameValue('\uD800\uDC00'.codePointAt(undefined), 65536);
assert.sameValue('\uD800\uDC00'.codePointAt([]), 65536);

assert.sameValue('\uD800\uDC00'.codePointAt('1'), 56320);
assert.sameValue('\uD800\uDC00'.codePointAt(true), 56320);
assert.sameValue('\uD800\uDC00'.codePointAt([1]), 56320);
