// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/
// Test that property keys are only evaluated once.

class PropertyKey {
  constructor(key) {
    this.key = key;
    this.count = 0;
  }

  toString() {
    this.count++;
    return this.key;
  }

  valueOf() {
    throw new Error("unexpected valueOf call");
  }
}

// AndAssignExpr
{
  let obj = {p: true};
  let pk = new PropertyKey("p");

  obj[pk] &&= false;

  assert.sameValue(obj.p, false);
  assert.sameValue(pk.count, 1);

  obj[pk] &&= true;

  assert.sameValue(obj.p, false);
  assert.sameValue(pk.count, 2);
}

// OrAssignExpr
{
  let obj = {p: false};
  let pk = new PropertyKey("p");

  obj[pk] ||= true;

  assert.sameValue(obj.p, true);
  assert.sameValue(pk.count, 1);

  obj[pk] ||= false;

  assert.sameValue(obj.p, true);
  assert.sameValue(pk.count, 2);
}

// CoalesceAssignExpr
{
  let obj = {p: null};
  let pk = new PropertyKey("p");

  obj[pk] ??= true;

  assert.sameValue(obj.p, true);
  assert.sameValue(pk.count, 1);

  obj[pk] ??= false;

  assert.sameValue(obj.p, true);
  assert.sameValue(pk.count, 2);
}

