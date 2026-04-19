// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
flags:
  - noStrict
description: |
  pending
esid: pending
---*/

function expectSyntaxError(str) {
  assert.throws(SyntaxError, function() {
    eval(str);
  });

  assert.throws(SyntaxError, function() {
    eval('"use strict";' + str);
  });
}

function expectSloppyPass(str) {
  eval(str);

  assert.throws(SyntaxError, function() {
    eval('"use strict";' + str);
  });
}

expectSloppyPass(`l: function f1() {}`);
expectSloppyPass(`l0: l: function f1() {}`);
expectSloppyPass(`{ f1(); l: function f1() {} }`);
expectSloppyPass(`{ f1(); l0: l: function f1() {} }`);
expectSloppyPass(`{ f1(); l: function f1() { return 42; } } assert.sameValue(f1(), 42);`);
expectSloppyPass(`eval("fe(); l: function fe() {}")`);
expectSyntaxError(`if (1) l: function f2() {}`);
expectSyntaxError(`if (1) {} else l: function f3() {}`);
expectSyntaxError(`do l: function f4() {} while (0)`);
expectSyntaxError(`while (0) l: function f5() {}`);
expectSyntaxError(`for (;;) l: function f6() {}`);
expectSloppyPass(`switch (1) { case 1: l: function f7() {} }`);
expectSloppyPass(`switch (1) { case 1: assert.sameValue(f8(), 'f8'); case 2: l: function f8() { return 'f8'; } } assert.sameValue(f8(), 'f8');`);
