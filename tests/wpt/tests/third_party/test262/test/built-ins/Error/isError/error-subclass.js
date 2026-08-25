// Copyright (C) 2024 Jordan Harband.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-error.iserror
description: >
  Returns true on userland Error subclasses
features: [Error.isError, class]
---*/

class MyError extends Error {}
class MyEvalError extends EvalError {}
class MyRangeError extends RangeError {}
class MyReferenceError extends ReferenceError {}
class MySyntaxError extends SyntaxError {}
class MyTypeError extends TypeError {}
class MyURIError extends URIError {}

assert.sameValue(Error.isError(new MyError()), true);
assert.sameValue(Error.isError(new MyEvalError()), true);
assert.sameValue(Error.isError(new MyRangeError()), true);
assert.sameValue(Error.isError(new MyReferenceError()), true);
assert.sameValue(Error.isError(new MySyntaxError()), true);
assert.sameValue(Error.isError(new MyTypeError()), true);
assert.sameValue(Error.isError(new MyURIError()), true);

if (typeof AggregateError !== 'undefined') {
  class MyAggregateError extends AggregateError {}

  assert.sameValue(Error.isError(new MyAggregateError([])), true);
}

if (typeof SuppressedError !== 'undefined') {
  class MySuppressedError extends SuppressedError {}

  assert.sameValue(Error.isError(new MySuppressedError()), true);
}
