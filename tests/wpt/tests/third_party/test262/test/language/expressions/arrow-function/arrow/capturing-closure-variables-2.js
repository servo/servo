// Copyright 2015 Microsoft Corporation. All rights reserved.
// This code is governed by the license found in the LICENSE file.

/*---
description: Capturing closure variables - with
es6id: 14.2
flags: [noStrict]
---*/

function foo(){
    var a = {a : 10};
    with(a){
        return () => a;
    }
 }

assert.sameValue(foo()(), 10, "Closure variable was captured incorrectly.");
