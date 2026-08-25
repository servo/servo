// Copyright (C) 2024 Jordan Harband.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-error.iserror
description: >
  Returns false on non-Error objects from a different realm
features: [Error.isError, cross-realm]
---*/

var other = $262.createRealm().global;

assert.sameValue(Error.isError(new other.Object()), false);
assert.sameValue(Error.isError(new other.Array()), false);
assert.sameValue(Error.isError(new other.Function('')), false);
assert.sameValue(Error.isError(new other.RegExp('a')), false);

assert.sameValue(Error.isError(other.Error), false);
assert.sameValue(Error.isError(other.EvalError), false);
assert.sameValue(Error.isError(other.RangeError), false);
assert.sameValue(Error.isError(other.ReferenceError), false);
assert.sameValue(Error.isError(other.SyntaxError), false);
assert.sameValue(Error.isError(other.TypeError), false);
assert.sameValue(Error.isError(other.URIError), false);

if (typeof other.AggregateError !== 'undefined') {
  assert.sameValue(Error.isError(other.AggregateError), false);
}
if (typeof other.SuppressedError !== 'undefined') {
  assert.sameValue(Error.isError(other.SuppressedError), false);
}
