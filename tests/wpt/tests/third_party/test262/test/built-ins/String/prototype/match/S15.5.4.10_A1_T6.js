// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: String.prototype.match (regexp)
es5id: 15.5.4.10_A1_T6
description: >
    Call match (regexp) function with x argument of new String object,
    where x is undefined variable
---*/

var __matched = new String("undefined").match(x);

var __expected = RegExp(x).exec("undefined");

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
if (__matched.length !== __expected.length) {
  throw new Test262Error('#1: __matched = new String("undefined").match(x); __expected = RegExp(x).exec("undefined"); __matched.length === __expected.length. Actual: ' + __matched.length);
}
//
//////////////////////////////////////////////////////////////////////////////

//////////////////////////////////////////////////////////////////////////////
//CHECK#2
if (__matched.index !== __expected.index) {
  throw new Test262Error('#2: __matched = new String("undefined").match(x); __expected = RegExp(x).exec("undefined"); __matched.index === __expected.index. Actual: ' + __matched.index);
}
//
//////////////////////////////////////////////////////////////////////////////

//////////////////////////////////////////////////////////////////////////////
//CHECK#3
if (__matched.input !== __expected.input) {
  throw new Test262Error('#3: __matched = new String("undefined").match(x); __expected = RegExp(x).exec("undefined"); __matched.input === __expected.input. Actual: ' + __matched.input);
}
//
//////////////////////////////////////////////////////////////////////////////

//////////////////////////////////////////////////////////////////////////////
//CHECK#4
for (var index = 0; index < __expected.length; index++) {
  if (__matched[index] !== __expected[index]) {
    throw new Test262Error('#4.' + index + ': __matched = new String("undefined").match(x); __expected = RegExp(x).exec("undefined"); __matched[' + index + ']===__expected[' + index + ']. Actual: ' + __matched[index]);
  }
}
//
//////////////////////////////////////////////////////////////////////////////

var x;
