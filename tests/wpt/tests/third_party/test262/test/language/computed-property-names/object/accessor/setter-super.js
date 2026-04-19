// Copyright (C) 2014 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 12.2.5
description: >
    computed property setters can call super methods
---*/

function ID(x) {
  return x;
}

var value;
var proto = {
  m(name, v) {
    value = name + ' ' + v;
  }
};
var object = {
  set ['a'](v) { super.m('a', v); },
  set [ID('b')](v) { super.m('b', v); },
  set [0](v) { super.m('0', v); },
  set [ID(1)](v) { super.m('1', v); },
};

Object.setPrototypeOf(object, proto);

object.a = 2;
assert.sameValue(value, 'a 2', "The value of `value` is `'a 2'`, after executing `object.a = 2;`");
object.b = 3;
assert.sameValue(value, 'b 3', "The value of `value` is `'b 3'`, after executing `object.b = 3;`");
object[0] = 4;
assert.sameValue(value, '0 4', "The value of `value` is `'0 4'`, after executing `object[0] = 4;`");
object[1] = 5;
assert.sameValue(value, '1 5', "The value of `value` is `'1 5'`, after executing `object[1] = 5;`");
