// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 10.4.3-1-4-s
description: this is not coerced to an object in strict mode (boolean)
flags: [noStrict]
---*/

function foo()
{
  'use strict';
  return typeof(this);
}

function bar()
{
  return typeof(this);
}

assert.sameValue(foo.call(true), 'boolean', 'foo.call(true)');
assert.sameValue(bar.call(true), 'object', 'bar.call(true)');
