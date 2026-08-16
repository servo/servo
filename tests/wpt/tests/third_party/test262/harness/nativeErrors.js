// Copyright (C) 2026 Jordan Harband. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: |
    Arrays of language-specified Error constructors, plus a helper that
    constructs a sample instance with appropriate arguments for the
    constructor's signature.

    `nativeErrors` contains every Error constructor whose first argument
    is a `message` string: %Error% and the six NativeErrors.

    `allErrorConstructors` additionally includes %AggregateError% and
    %SuppressedError% when present in the host. Their constructors have
    different signatures (`(errors, message)` and
    `(error, suppressed, message)` respectively), so tests that just
    iterate as `new Ctor(message)` should prefer `nativeErrors`; tests
    that need to cover every Error-like constructor should use
    `allErrorConstructors` together with `makeNativeError`.
defines: [nativeErrors, allErrorConstructors, makeNativeError]
---*/

var nativeErrors = [
  Error,
  EvalError,
  RangeError,
  ReferenceError,
  SyntaxError,
  TypeError,
  URIError
];

var allErrorConstructors = nativeErrors.slice();
if (typeof AggregateError !== 'undefined') {
  allErrorConstructors.push(AggregateError);
}
if (typeof SuppressedError !== 'undefined') {
  allErrorConstructors.push(SuppressedError);
}

function makeNativeError(Ctor, useNew) {
  if (typeof AggregateError !== 'undefined' && Ctor === AggregateError) {
    return useNew
      ? new AggregateError([new Error('inner')], 'msg')
      : AggregateError([new Error('inner')], 'msg');
  }
  if (typeof SuppressedError !== 'undefined' && Ctor === SuppressedError) {
    return useNew
      ? new SuppressedError(new Error('inner'), new Error('suppressed'), 'msg')
      : SuppressedError(new Error('inner'), new Error('suppressed'), 'msg');
  }
  return useNew ? new Ctor('msg') : Ctor('msg');
}
