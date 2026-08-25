// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-super-keyword
description: SuperCall should evaluate Arguments prior to checking IsConstructor
info: |
  SuperCall : `super` Arguments

  [...]
  3. Let _func_ be ! GetSuperConstructor().
  4. Let _argList_ be ? ArgumentListEvaluation of |Arguments|.
  5. If IsConstructor(_func_) is *false*, throw a *TypeError* exception.
  [...]
features: [class]
---*/

var evaluatedArg = false;
var caught;
class C extends Object {
  constructor() {
    try {
      super(evaluatedArg = true);
    } catch (err) {
      caught = err;
    }
  }
}

Object.setPrototypeOf(C, parseInt);

// When the "construct" invocation completes and the "this" value is
// uninitialized, the specification dictates that a ReferenceError must be
// thrown. That behavior is tested elsewhere, so the error is ignored (if it is
// produced at all).
try {
  new C();
} catch (_) {}

assert.sameValue(typeof caught, 'object');
assert.sameValue(caught.constructor, TypeError);
assert(evaluatedArg, 'performs ArgumentsListEvaluation');
