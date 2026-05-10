// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 10.6-13-a-3
description: An indirect call to arguments.callee.caller should work
flags: [noStrict]
features: [caller]
---*/

var called = false;

function test1(flag) {
    if (flag!==true) {
        test2();
    } else {
        called = true;
    }
}

function test2() {
   if (arguments.callee.caller===undefined) {
     called = true;  //Extension not supported - fake it
   } else {
     var explicit = arguments.callee.caller;
     explicit(true);
   }
}

test1();

assert(called, 'called !== true');
