// Copyright 2018 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.locale
description: >
  The value of the [[Prototype]] internal slot of the Intl.Locale constructor is the
  intrinsic object %FunctionPrototype%.
features: [Intl.Locale]
---*/

assert.sameValue(
  Object.getPrototypeOf(Intl.Locale),
  Function.prototype,
  "Object.getPrototypeOf(Intl.Locale) equals the value of Function.prototype"
);
