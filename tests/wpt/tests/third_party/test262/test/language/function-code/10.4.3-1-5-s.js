// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 10.4.3-1-5-s
description: this is not coerced to an object (function)
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

function foobar()
{
}

assert.sameValue(foo.call(foobar), 'function', 'foo.call(foobar)');
assert.sameValue(bar.call(foobar), 'function', 'bar.call(foobar)');
