// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Boolean coercion of `global` property
esid: sec-regexp.prototype-@@match
info: |
    21.2.5.6 RegExp.prototype [ @@match ] ( string )

    [...]
    4. Let global be ToBoolean(? Get(rx, "global")).
    5. If global is false, then
       a. Return ? RegExpExec(rx, S).
    6. Else global is true,
       a. Let fullUnicode be ToBoolean(? Get(rx, "unicode")).
       [...]
features: [Symbol.match]
---*/

var exec = function() {
  execCount += 1;
  if (execCount === 1) {
    return [''];
  }
  return null;
};
var r, result, execCount;

r = /a/g;
r.exec = exec;
Object.defineProperty(r, 'global', { writable: true });

execCount = 0;
r.global = undefined;
r[Symbol.match]('aa');
assert.sameValue(execCount, 1, 'value: undefined');

execCount = 0;
r.global = null;
r[Symbol.match]('aa');
assert.sameValue(execCount, 1, 'value: null');

execCount = 0;
r.global = false;
r[Symbol.match]('aa');
assert.sameValue(execCount, 1, 'value: false');

execCount = 0;
r.global = NaN;
r[Symbol.match]('aa');
assert.sameValue(execCount, 1, 'value: NaN');

execCount = 0;
r.global = 0;
r[Symbol.match]('aa');
assert.sameValue(execCount, 1, 'value: 0');

execCount = 0;
r.global = '';
r[Symbol.match]('aa');
assert.sameValue(execCount, 1, 'value: ""');

r = /a/;
r.exec = exec;
Object.defineProperty(r, 'global', { writable: true });

r.global = true;
execCount = 0;
r[Symbol.match]('aa');
assert.sameValue(execCount, 2, 'value: true');

r.global = 86;
execCount = 0;
r[Symbol.match]('aa');
assert.sameValue(execCount, 2, 'value: 86');

r.global = Symbol.match;
execCount = 0;
r[Symbol.match]('aa');
assert.sameValue(execCount, 2, 'value: Symbol.match');

r.global = {};
execCount = 0;
r[Symbol.match]('aa');
assert.sameValue(execCount, 2, 'value: {}');
