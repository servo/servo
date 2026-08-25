// Copyright (C) 2020 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.datetimeformat.prototype-@@tostringtag
description: >
  Object.prototype.toString utilizes Intl.DateTimeFormat.prototype[@@toStringTag].
info: |
  Object.prototype.toString ( )

  [...]
  14. Else, let builtinTag be "Object".
  15. Let tag be ? Get(O, @@toStringTag).
  16. If Type(tag) is not String, set tag to builtinTag.
  17. Return the string-concatenation of "[object ", tag, and "]".

  Intl.DateTimeFormat.prototype [ @@toStringTag ]

  This property has the attributes { [[Writable]]: false, [[Enumerable]]: false, [[Configurable]]: true }.
features: [Symbol.toStringTag]
---*/

Object.defineProperty(Intl.DateTimeFormat.prototype, Symbol.toStringTag, {
  value: "test262",
});

assert.sameValue(Object.prototype.toString.call(Intl.DateTimeFormat.prototype), "[object test262]");
assert.sameValue(Object.prototype.toString.call(new Intl.DateTimeFormat()), "[object test262]");
