// Copyright (C) 2019 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-json.parse
description: >
  Requirements for built-in functions, defined in introduction of chapter 17,
  are satisfied.
features: [Reflect.construct]
---*/

var parse = JSON.parse;
assert(Object.isExtensible(parse), 'Object.isExtensible(parse) must return true');
assert.sameValue(typeof parse, 'function', 'The value of `typeof parse` is "function"');
assert.sameValue(
  Object.prototype.toString.call(parse),
  '[object Function]',
  'Object.prototype.toString.call("JSON.parse") must return "[object Function]"'
);
assert.sameValue(
  Object.getPrototypeOf(parse),
  Function.prototype,
  'Object.getPrototypeOf("JSON.parse") must return the value of Function.prototype'
);
assert.sameValue(
  parse.hasOwnProperty('prototype'),
  false,
  'parse.hasOwnProperty("prototype") must return false'
);
