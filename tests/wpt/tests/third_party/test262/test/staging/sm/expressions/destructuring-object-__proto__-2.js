// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/
// Test __proto__ is destructuring assignment.

// __proto__ shorthand, no default.
{
  let __proto__;
  ({__proto__} = {});
  assert.sameValue(__proto__, Object.prototype);
}

// __proto__ shorthand, with default.
{
  let __proto__;
  ({__proto__ = 0} = {});
  assert.sameValue(__proto__, Object.prototype);
}

{
  let __proto__;
  ({__proto__ = 0} = Object.create(null));
  assert.sameValue(__proto__, 0);
}

// __proto__ keyed, no default.
{
  let p;
  ({__proto__: p} = {});
  assert.sameValue(p, Object.prototype);
}

// __proto__ keyed, with default.
{
  let p;
  ({__proto__: p = 0} = {});
  assert.sameValue(p, Object.prototype);
}

// __proto__ keyed, with default.
{
  let p;
  ({__proto__: p = 0} = Object.create(null));
  assert.sameValue(p, 0);
}

// Repeat the cases from above, but this time with a rest property.

// __proto__ shorthand, no default.
{
  let __proto__, rest;
  ({__proto__, ...rest} = {});
  assert.sameValue(__proto__, Object.prototype);
  assert.sameValue(Reflect.ownKeys(rest).length, 0);
}

// __proto__ shorthand, with default.
{
  let __proto__, rest;
  ({__proto__ = 0, ...rest} = {});
  assert.sameValue(__proto__, Object.prototype);
  assert.sameValue(Reflect.ownKeys(rest).length, 0);
}

{
  let __proto__, rest;
  ({__proto__ = 0, ...rest} = Object.create(null));
  assert.sameValue(__proto__, 0);
  assert.sameValue(Reflect.ownKeys(rest).length, 0);
}

// __proto__ keyed, no default.
{
  let p, rest;
  ({__proto__: p, ...rest} = {});
  assert.sameValue(p, Object.prototype);
  assert.sameValue(Reflect.ownKeys(rest).length, 0);
}

// __proto__ keyed, with default.
{
  let p, rest;
  ({__proto__: p = 0, ...rest} = {});
  assert.sameValue(p, Object.prototype);
  assert.sameValue(Reflect.ownKeys(rest).length, 0);
}

// __proto__ keyed, with default.
{
  let p, rest;
  ({__proto__: p = 0, ...rest} = Object.create(null));
  assert.sameValue(p, 0);
  assert.sameValue(Reflect.ownKeys(rest).length, 0);
}

