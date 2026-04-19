// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Boolean coercion of `global` property
es6id: 21.2.5.8
info: |
    21.2.5.6 RegExp.prototype [ @@replace ] ( string )

    [...]
    8. Let global be ToBoolean(Get(rx, "global")).
    [...]
features: [Symbol.replace]
---*/

Array.print = print;
var r = /a/g;
Object.defineProperty(r, 'global', { writable: true });

r.lastIndex = 0;
r.global = undefined;
assert.sameValue(r[Symbol.replace]('aa', 'b'), 'ba', 'value: undefined');

r.lastIndex = 0;
r.global = null;
assert.sameValue(r[Symbol.replace]('aa', 'b'), 'ba', 'value: null');

r.lastIndex = 0;
r.global = false;
assert.sameValue(r[Symbol.replace]('aa', 'b'), 'ba', 'value: false');

r.lastIndex = 0;
r.global = NaN;
assert.sameValue(r[Symbol.replace]('aa', 'b'), 'ba', 'value: NaN');

r.lastIndex = 0;
r.global = 0;
assert.sameValue(r[Symbol.replace]('aa', 'b'), 'ba', 'value: global');

r.lastIndex = 0;
r.global = '';
assert.sameValue(r[Symbol.replace]('aa', 'b'), 'ba', 'value: ""');

var execCount = 0;
r = /a/;
Object.defineProperty(r, 'global', { writable: true });
r.exec = function() {
  execCount += 1;
  if (execCount === 1) {
    return ['a'];
  }
  return null;
};

execCount = 0;
r.global = true;
r[Symbol.replace]('aa', 'b');
assert.sameValue(execCount, 2, 'value: true');

execCount = 0;
r.global = 86;
r[Symbol.replace]('aa', 'b');
assert.sameValue(execCount, 2, 'value: 86');

execCount = 0;
r.global = Symbol.replace;
r[Symbol.replace]('aa', 'b');
assert.sameValue(execCount, 2, 'value: Symbol.replace');

execCount = 0;
r.global = {};
r[Symbol.replace]('aa', 'b');
assert.sameValue(execCount, 2, 'value: {}');
