// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es6id: 21.2.5.9
description: >
    The `lastIndex` value is set to 0 immediately prior to match execution
info: |
    [...]
    7. Let status be Set(rx, "lastIndex", 0, true).
    8. ReturnIfAbrupt(status).
    9. Let result be RegExpExec(rx, S).
    [...]
features: [Symbol.search]
---*/

var duringExec;
var fakeRe = {
  lastIndex: 34,
  exec: function() {
    duringExec = fakeRe.lastIndex;
    return null;
  }
};

RegExp.prototype[Symbol.search].call(fakeRe);

assert.sameValue(duringExec, 0);
