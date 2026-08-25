// Copyright (C) 2020 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl-toStringTag
description: >
  Object.prototype.toString utilizes Intl[@@toStringTag] and doesn't special-case Intl namespace object.
info: |
  Object.prototype.toString ( )

  [...]
  14. Else, let builtinTag be "Object".
  15. Let tag be ? Get(O, @@toStringTag).
  16. If Type(tag) is not String, set tag to builtinTag.
  17. Return the string-concatenation of "[object ", tag, and "]".

  Intl [ @@toStringTag ]

  The initial value of the @@toStringTag property is the String value "Intl".
features: [Symbol.toStringTag]
---*/

assert.sameValue(Intl.toString(), "[object Intl]");
assert.sameValue(Object.prototype.toString.call(Intl), "[object Intl]");

Object.defineProperty(Intl, Symbol.toStringTag, { value: "test262" });
assert.sameValue(Intl.toString(), "[object test262]");
assert.sameValue(Object.prototype.toString.call(Intl), "[object test262]");

assert(delete Intl[Symbol.toStringTag]);
assert.sameValue(Intl.toString(), "[object Object]");
assert.sameValue(Object.prototype.toString.call(Intl), "[object Object]");
