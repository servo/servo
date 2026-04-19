// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/
// Test TDZ for optional chaining.

// TDZ for lexical |let| bindings with optional chaining.
{
  assert.throws(ReferenceError, () => {
    const Null = null;
    Null?.[b];
    b = 0;
    let b;
  });

  assert.throws(ReferenceError, () => {
    const Null = null;
    Null?.[b]();
    b = 0;
    let b;
  });

  assert.throws(ReferenceError, () => {
    const Null = null;
    delete Null?.[b];
    b = 0;
    let b;
  });
}

