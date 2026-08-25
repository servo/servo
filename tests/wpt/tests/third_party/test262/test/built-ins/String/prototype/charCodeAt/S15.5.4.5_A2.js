// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    When String.prototype.charCodeAt(pos) calls if ToInteger(pos) less than 0
    the NaN returns
es5id: 15.5.4.5_A2
description: Call charCodeAt(pos) with negative pos
---*/

function __FACTORY() {};

__FACTORY.prototype.charCodeAt = String.prototype.charCodeAt;

var __instance = new __FACTORY;

assert.sameValue(__instance.charCodeAt(-1), NaN);
