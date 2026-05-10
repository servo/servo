// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 10.4.3-1-2-s
description: this is not coerced to an object in strict mode (string)
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

assert.sameValue(foo.call('1'), 'string', 'foo.call("1")');
assert.sameValue(bar.call('1'), 'object', 'bar.call("1")');
