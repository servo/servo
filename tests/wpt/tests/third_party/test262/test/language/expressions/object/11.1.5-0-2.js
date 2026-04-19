// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    it isn't clear what specific requirements of the specificaiton are being tested here. This test should 
    probably be replaced by some more targeted tests.  AllenWB
es5id: 11.1.5-0-2
description: Object literal - multiple get set properties
---*/

  var s1 = "First getter";
  var s2 = "First setter";
  var s3 = "Second getter";
  var o;
  eval("o = {get foo(){ return s1;},set foo(arg){return s2 = s3}, get bar(){ return s3}, set bar(arg){ s3 = arg;}};");

assert.sameValue(o.foo, s1, 'o.foo');

  o.foo = 10;

assert.sameValue(s2, s3, 's2');
assert.sameValue(o.bar, s3, 'o.bar');

  o.bar = "Second setter";

assert.sameValue(o.bar, "Second setter", 'o.bar');
