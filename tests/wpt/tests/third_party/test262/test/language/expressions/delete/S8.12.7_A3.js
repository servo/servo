// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    When the [[Delete]] method of O is called with property name P,
    removes the property with name P from O and return true
esid: sec-delete-operator-runtime-semantics-evaluation
description: Delete existent properties
---*/

var BLUE_NUM = 1;
var BLUE_STR = '1';
var YELLOW_NUM = 2;
var YELLOW_STR = '2';
var __color__map = {
  red: 0xff0000,
  BLUE_NUM: 0x0000ff,
  green: 0x00ff00,
  YELLOW_STR: 0xffff00,
};

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
if (delete __color__map[YELLOW_NUM] !== true) {
  throw new Test262Error(
    '#1: var BLUE_NUM=1; var BLUE_STR="1"; var YELLOW_NUM=2; var YELLOW_STR="2"; var __color__map = {red:0xFF0000, BLUE_NUM:0x0000FF, green:0x00FF00, YELLOW_STR:0xFFFF00}; delete __color__map[YELLOW_NUM] === true;'
  );
}
//
//////////////////////////////////////////////////////////////////////////////

//////////////////////////////////////////////////////////////////////////////
//CHECK#2
if (__color__map[YELLOW_STR] !== undefined) {
  throw new Test262Error(
    '#2: var BLUE_NUM=1; var BLUE_STR="1"; var YELLOW_NUM=2; var YELLOW_STR="2"; var __color__map = {red:0xFF0000, BLUE_NUM:0x0000FF, green:0x00FF00, YELLOW_STR:0xFFFF00}; delete __color__map[YELLOW_NUM]; __color__map[YELLOW_STR] === undefined. Actual: ' +
    __color__map[YELLOW_STR]
  );
}
//
//////////////////////////////////////////////////////////////////////////////

//////////////////////////////////////////////////////////////////////////////
//CHECK#3
if (delete __color__map[BLUE_STR] !== true) {
  throw new Test262Error(
    '#3: var BLUE_NUM=1; var BLUE_STR="1"; var YELLOW_NUM=2; var YELLOW_STR="2"; var __color__map = {red:0xFF0000, BLUE_NUM:0x0000FF, green:0x00FF00, YELLOW_STR:0xFFFF00}; delete __color__map[BLUE_STR] === true. Actual: ' +
    delete __color__map[BLUE_STR]
  );
}
//
//////////////////////////////////////////////////////////////////////////////

//////////////////////////////////////////////////////////////////////////////
//CHECK#4
if (__color__map[BLUE_NUM] !== undefined) {
  throw new Test262Error(
    '#4: var BLUE_NUM=1; var BLUE_STR="1"; var YELLOW_NUM=2; var YELLOW_STR="2"; var __color__map = {red:0xFF0000, BLUE_NUM:0x0000FF, green:0x00FF00, YELLOW_STR:0xFFFF00}; delete __color__map[BLUE_STR]; __color__map[BLUE_NUM] === undefined. Actual: ' +
    __color__map[BLUE_NUM]
  );
}
//
//////////////////////////////////////////////////////////////////////////////
