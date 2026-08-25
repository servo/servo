// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-regexp-pattern-flags
description: Initial state of the `lastIndex` property
info: |
  [...]
  7. Let O be ? RegExpAlloc(newTarget).
  8. Return ? RegExpInitialize(O, P, F).

  21.2.3.2.2 Runtime Semantics: RegExpInitialize

  [...]
  12. Perform ? Set(obj, "lastIndex", 0, true).
  [...]

  21.2.3.2.1 Runtime Semantics: RegExpAlloc

  [...]
  2. Perform ! DefinePropertyOrThrow(obj, "lastIndex", PropertyDescriptor
     {[[Writable]]: true, [[Enumerable]]: false, [[Configurable]]: false}).
  [...]
includes: [propertyHelper.js]
---*/

var re = new RegExp('');

assert.sameValue(re.lastIndex, 0);

verifyProperty(re, 'lastIndex', {
  writable: true,
  enumerable: false,
  configurable: false,
});
