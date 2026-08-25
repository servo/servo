// Copyright (C) 2017 Valerie Young. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-string.prototype.trimLeft
description: >
  String.prototype.trimLeft is a reference to String.prototype.trimStart.
info: >
  String.prototype.trimLeft ( )

  The function object that is the initial value of String.prototype.trimLeft
  is the same function object that is the initial value of
  String.prototype.trimStart.
features: [string-trimming]
---*/

assert.sameValue(String.prototype.trimLeft, String.prototype.trimStart);
