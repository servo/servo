// Copyright (C) 2024 Jordan Harband.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-error.iserror
description: >
  Returns true on Error and Error subclass instances from a different realm
features: [Error.isError, cross-realm]
---*/

var other = $262.createRealm().global;

assert.sameValue(Error.isError(new other.Error()), true);
assert.sameValue(Error.isError(new other.EvalError()), true);
assert.sameValue(Error.isError(new other.RangeError()), true);
assert.sameValue(Error.isError(new other.ReferenceError()), true);
assert.sameValue(Error.isError(new other.SyntaxError()), true);
assert.sameValue(Error.isError(new other.TypeError()), true);
assert.sameValue(Error.isError(new other.URIError()), true);

if (typeof AggregateError !== 'undefined') {
  assert.sameValue(Error.isError(new other.AggregateError([])), true);
}
if (typeof SuppressedError !== 'undefined') {
  assert.sameValue(Error.isError(new other.SuppressedError()), true);
}
