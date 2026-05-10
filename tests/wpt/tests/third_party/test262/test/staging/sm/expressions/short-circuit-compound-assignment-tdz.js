// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/
// Test TDZ for short-circuit compound assignments.

// TDZ for lexical |let| bindings.
{
  assert.throws(ReferenceError, () => { let a = (a &&= 0); });
  assert.throws(ReferenceError, () => { let a = (a ||= 0); });
  assert.throws(ReferenceError, () => { let a = (a ??= 0); });
}

// TDZ for lexical |const| bindings.
{
  assert.throws(ReferenceError, () => { const a = (a &&= 0); });
  assert.throws(ReferenceError, () => { const a = (a ||= 0); });
  assert.throws(ReferenceError, () => { const a = (a ??= 0); });
}

// TDZ for parameter expressions.
{
  assert.throws(ReferenceError, (a = (b &&= 0), b) => {});
  assert.throws(ReferenceError, (a = (b ||= 0), b) => {});
  assert.throws(ReferenceError, (a = (b ??= 0), b) => {});
}

// TDZ for |class| bindings.
{
  assert.throws(ReferenceError, () => { class a extends (a &&= 0) {} });
  assert.throws(ReferenceError, () => { class a extends (a ||= 0) {} });
  assert.throws(ReferenceError, () => { class a extends (a ??= 0) {} });
}

// TDZ for lexical |let| bindings with conditional assignment.
{
  assert.throws(ReferenceError, () => {
    const False = false;
    False &&= b;
    b = 2;
    let b;
  });

  assert.throws(ReferenceError, () => {
    const True = true;
    True ||= b;
    b = 2;
    let b;
  });

  assert.throws(ReferenceError, () => {
    const NonNull = {};
    NonNull ??= b;
    b = 2;
    let b;
  });
}

