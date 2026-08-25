// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
includes: [compareArray.js]
description: |
  pending
esid: pending
---*/
assert.sameValue(typeof AggregateError, "function");
assert.sameValue(Object.getPrototypeOf(AggregateError), Error);
assert.sameValue(AggregateError.name, "AggregateError");
assert.sameValue(AggregateError.length, 2);

assert.sameValue(Object.getPrototypeOf(AggregateError.prototype), Error.prototype);
assert.sameValue(AggregateError.prototype.name, "AggregateError");
assert.sameValue(AggregateError.prototype.message, "");

// The |errors| argument is mandatory.
assert.throws(TypeError, () => new AggregateError());
assert.throws(TypeError, () => AggregateError());

// The .errors data property is an array object.
{
  let err = new AggregateError([]);

  let {errors} = err;
  assert.sameValue(Array.isArray(errors), true);
  assert.sameValue(errors.length, 0);

  // The errors object is modifiable.
  errors.push(123);
  assert.sameValue(errors.length, 1);
  assert.sameValue(errors[0], 123);
  assert.sameValue(err.errors[0], 123);

  // The property is writable.
  err.errors = undefined;
  assert.sameValue(err.errors, undefined);
}

// The errors argument can be any iterable.
{
  function* g() { yield* [1, 2, 3]; }

  let {errors} = new AggregateError(g());
  assert.compareArray(errors, [1, 2, 3]);
}

// The message property is populated by the second argument.
{
  let err;

  err = new AggregateError([]);
  assert.sameValue(err.message, "");

  err = new AggregateError([], "my message");
  assert.sameValue(err.message, "my message");
}

{
  assert.sameValue("errors" in AggregateError.prototype, false);

  const {
    configurable,
    enumerable,
    value,
    writable
  } = Object.getOwnPropertyDescriptor(new AggregateError([]), "errors");
  assert.sameValue(configurable, true);
  assert.sameValue(enumerable, false);
  assert.sameValue(writable, true);
  assert.sameValue(value.length, 0);

  const g = $262.createRealm().global;

  let obj = {};
  let errors = new g.AggregateError([obj]).errors;

  assert.sameValue(errors.length, 1);
  assert.sameValue(errors[0], obj);

  // The prototype is |g.Array.prototype| in the cross-compartment case.
  let proto = Object.getPrototypeOf(errors);
  assert.sameValue(proto === g.Array.prototype, true);
}

