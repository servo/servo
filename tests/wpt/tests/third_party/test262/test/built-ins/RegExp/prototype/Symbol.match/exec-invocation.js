// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Invocation of `exec` method
es6id: 21.2.5.6
info: |
    [...]
    7. If global is false, then
       a. Return RegExpExec(rx, S).

    21.2.5.2.1 Runtime Semantics: RegExpExec ( R, S )

    [...]
    5. If IsCallable(exec) is true, then
       a. Let result be Call(exec, R, «S»).
features: [Symbol.match]
---*/

var r = /./;
var callCount = 0;
var arg = {
  toString: function() {
    return 'string form';
  }
};
var thisValue, args;

r.exec = function() {
  thisValue = this;
  args = arguments;
  callCount += 1;
  return null;
};

r[Symbol.match](arg);

assert.sameValue(callCount, 1);
assert.sameValue(thisValue, r);
assert.sameValue(args.length, 1);
assert.sameValue(args[0], 'string form');
