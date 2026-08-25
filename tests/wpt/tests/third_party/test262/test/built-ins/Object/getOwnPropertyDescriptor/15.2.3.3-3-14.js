// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.3-3-14
description: >
    Object.getOwnPropertyDescriptor applied to a String object which
    implements its own property get method
---*/

var str = new String("123");

var desc = Object.getOwnPropertyDescriptor(str, "2");

assert.sameValue(desc.value, "3", 'desc.value');
