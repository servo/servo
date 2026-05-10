// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 8.12.9-9-c-i_1
description: >
    Redefine a configurable accessor property to be a data property on
    a non-extensible object
---*/

var o = {};
Object.defineProperty(o, "foo",
{
  get: function() {
    return 5;
  },
  configurable: true
});
Object.preventExtensions(o);
Object.defineProperty(o, "foo", {
  value: "hello"
});

var fooDescrip = Object.getOwnPropertyDescriptor(o, "foo");

assert.sameValue(o.foo, "hello", 'o.foo');
assert.sameValue(fooDescrip.get, undefined, 'fooDescrip.get');
assert.sameValue(fooDescrip.set, undefined, 'fooDescrip.set');
assert.sameValue(fooDescrip.value, "hello", 'fooDescrip.value');
assert.sameValue(fooDescrip.configurable, true, 'fooDescrip.configurable');
assert.sameValue(fooDescrip.enumerable, false, 'fooDescrip.enumerable');
assert.sameValue(fooDescrip.writable, false, 'fooDescrip.writable');
