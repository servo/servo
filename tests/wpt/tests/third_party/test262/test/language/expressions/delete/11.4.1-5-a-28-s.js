// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-delete-operator-runtime-semantics-evaluation
description: Strict Mode - TypeError is not thrown when deleting RegExp.length
---*/

var a = new RegExp();
var b = delete RegExp.length;
