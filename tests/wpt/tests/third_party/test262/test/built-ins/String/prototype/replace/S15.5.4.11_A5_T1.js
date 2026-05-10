// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Use replace with regexp as searchValue and use $ in replaceValue
es5id: 15.5.4.11_A5_T1
description: searchValue is  regexp /^(a+)\1*,\1+$/ and replaceValue is "$1"
---*/

var __str = "aaaaaaaaaa,aaaaaaaaaaaaaaa";
var __pattern = /^(a+)\1*,\1+$/;
var __repl = "$1";

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
if (__str.replace(__pattern, __repl) !== 'aaaaa') {
  throw new Test262Error('#1: var __str = "aaaaaaaaaa,aaaaaaaaaaaaaaa"; var __pattern = /^(a+)\\1*,\\1+$/; var __repl = "$1"; __str.replace(__pattern, __repl)===\'aaaaa\'. Actual: ' + __str.replace(__pattern, __repl));
}
//
//////////////////////////////////////////////////////////////////////////////
