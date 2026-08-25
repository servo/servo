// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 10.2.1.1.3-4-22-s
description: >
  TypeError is not thrown when changing the value of the Constructor Properties
  of the Global Object
---*/

var objBak = Object;

try {
  Object = 12;
} finally {
  Object = objBak;
}
