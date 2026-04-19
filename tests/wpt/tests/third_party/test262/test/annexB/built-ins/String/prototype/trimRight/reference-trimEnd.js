// Copyright (C) 2017 Valerie Young. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-string.prototype.trimRight
description: >
  String.prototype.trimRight is a reference to String.prototype.trimEnd.
info: >
  String.prototype.trimRight ( )

  The function object that is the initial value of String.prototype.trimRight
  is the same function object that is the initial value of
  String.prototype.trimEnd.
features: [string-trimming]
---*/

assert.sameValue(String.prototype.trimRight, String.prototype.trimEnd);
