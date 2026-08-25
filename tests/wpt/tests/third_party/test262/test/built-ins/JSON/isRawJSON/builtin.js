// Copyright (C) 2024 Igalia S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-json.israwjson
description: >
  JSON.isRawJSON meets the requirements for built-in objects
info: |
  JSON.isRawJSON ( O )

  18 ECMAScript Standard Built-in Objects
  ...
  Unless specified otherwise, a built-in object that is callable as a function
  is a built-in function object with the characteristics described in 10.3.
  Unless specified otherwise, the [[Extensible]] internal slot of a built-in
  object initially has the value true. Every built-in function object has a
  [[Realm]] internal slot whose value is the Realm Record of the realm for
  which the object was initially created.
  ...
  Unless otherwise specified every built-in function and every built-in
  constructor has the Function prototype object, which is the initial value of
  the expression Function.prototype (20.2.3), as the value of its [[Prototype]]
  internal slot.
  ...
  Built-in function objects that are not identified as constructors do not
  implement the [[Construct]] internal method unless otherwise specified in the
  description of a particular function.

features: [json-parse-with-source]
---*/

assert(Object.isExtensible(JSON.isRawJSON), "JSON.isRawJSON is extensible");
assert.sameValue(
  typeof JSON.isRawJSON,
  'function',
  'The value of `typeof JSON.isRawJSON` is "function"'
);
assert.sameValue(
  Object.getPrototypeOf(JSON.isRawJSON),
  Function.prototype,
  'Object.getPrototypeOf(JSON.isRawJSON) must return the value of "Function.prototype"'
);

assert.sameValue(
  Object.getOwnPropertyDescriptor(JSON.isRawJSON, "prototype"),
  undefined,
  "JSON.isRawJSON has no own prototype property"
);
