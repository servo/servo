// Copyright (C) 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-super-keyword
description: Binds the "this" value to value returned by "parent" constructor
info: |
  6. Let result be ? Construct(func, argList, newTarget).
  7. Let thisER be GetThisEnvironment( ).
  8. Return ? thisER.BindThisValue(result).
features: [class]
---*/

class X {
  method() { return this; }
}

class Y extends X {
  method() { return super.method(); }
}

const y = new Y();

assert.sameValue(y.method(), y);
