// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/
// getter/setter with expression closure is allowed only in object literal.

assert.throws(SyntaxError, () => eval(`
  class foo {
    constructor() {}

    get a() 1
  }
`));

assert.throws(SyntaxError, () => eval(`
  class foo {
    constructor() {}

    set a(v) 1
  }
`));

