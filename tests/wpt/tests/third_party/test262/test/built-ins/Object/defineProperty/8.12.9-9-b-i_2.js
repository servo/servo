// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 8.12.9-9-b-i_2
description: >
    Redefine a configurable data property to be an accessor property
    on a newly non-extensible object
---*/

var o = {};
Object.defineProperty(o, "foo",
{
  value: "hello",
  configurable: true,
  enumerable: true,
  writable: true
});
Object.preventExtensions(o);
Object.defineProperty(o, "foo", {
  get: function() {
    return 5;
  }
});

var fooDescrip = Object.getOwnPropertyDescriptor(o, "foo");

assert.sameValue(o.foo, 5, 'o.foo');
assert(fooDescrip.get !== undefined, 'fooDescrip.get!==undefined !== true');
assert.sameValue(fooDescrip.set, undefined, 'fooDescrip.set');
assert.sameValue(fooDescrip.value, undefined, 'fooDescrip.value');
assert.sameValue(fooDescrip.configurable, true, 'fooDescrip.configurable');
assert.sameValue(fooDescrip.enumerable, true, 'fooDescrip.enumerable');
assert.sameValue(fooDescrip.writable, undefined, 'fooDescrip.writable');
