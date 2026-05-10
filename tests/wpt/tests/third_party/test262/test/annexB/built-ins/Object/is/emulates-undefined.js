// Copyright (C) 2020 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-object.is
description: >
  SameValue abstract op doesn't special-case [[IsHTMLDDA]] objects.
info: |
  Object.is ( value1, value2 )

  1. Return SameValue(value1, value2).

  SameValue ( x, y )

  1. If Type(x) is different from Type(y), return false.
features: [IsHTMLDDA]
---*/

var IsHTMLDDA = $262.IsHTMLDDA;

assert.sameValue(Object.is(IsHTMLDDA, undefined), false, "SameValue with `undefined`");
assert.sameValue(Object.is(undefined, IsHTMLDDA), false, "SameValue with `undefined`");

assert.sameValue(Object.is(IsHTMLDDA, null), false, "SameValue with `null`");
assert.sameValue(Object.is(null, IsHTMLDDA), false, "SameValue with `null`");

assert(Object.is(IsHTMLDDA, IsHTMLDDA));
