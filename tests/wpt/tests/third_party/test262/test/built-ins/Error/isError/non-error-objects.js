// Copyright (C) 2024 Jordan Harband.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-error.iserror
description: >
  Returns false on non-Error objects
features: [Error.isError]
---*/

assert.sameValue(Error.isError({}), false);
assert.sameValue(Error.isError([]), false);
assert.sameValue(Error.isError(function () {}), false);
assert.sameValue(Error.isError(/a/g), false);

assert.sameValue(Error.isError(Error), false);
assert.sameValue(Error.isError(EvalError), false);
assert.sameValue(Error.isError(RangeError), false);
assert.sameValue(Error.isError(ReferenceError), false);
assert.sameValue(Error.isError(SyntaxError), false);
assert.sameValue(Error.isError(TypeError), false);
assert.sameValue(Error.isError(URIError), false);

if (typeof AggregateError !== 'undefined') {
  assert.sameValue(Error.isError(AggregateError), false);
}
if (typeof SuppressedError !== 'undefined') {
  assert.sameValue(Error.isError(SuppressedError), false);
}
