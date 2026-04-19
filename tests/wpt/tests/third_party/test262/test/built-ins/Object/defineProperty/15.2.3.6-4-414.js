// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-414
description: >
    ES5 Attributes - Inherited property whose [[Enumerable]] attribute
    is set to true is enumerable (Object.create)
---*/

var appointment = new Object();

Object.defineProperty(appointment, "startTime", {
  value: 1001,
  writable: true,
  enumerable: true,
  configurable: true
});
Object.defineProperty(appointment, "name", {
  value: "NAME",
  writable: true,
  enumerable: true,
  configurable: true
});

var meeting = Object.create(appointment);
Object.defineProperty(meeting, "conferenceCall", {
  value: "In-person meeting",
  writable: true,
  enumerable: true,
  configurable: true
});

var teamMeeting = Object.create(meeting);

var verifyTimeProp = false;
var verifyNameProp = false;
var verifyCallProp = false;
for (var p in teamMeeting) {
  if (p === "startTime") {
    verifyTimeProp = true;
  }
  if (p === "name") {
    verifyNameProp = true;
  }
  if (p === "conferenceCall") {
    verifyCallProp = true;
  }
}

var hasOwnProperty = !teamMeeting.hasOwnProperty("name") &&
  !teamMeeting.hasOwnProperty("startTime") &&
  !teamMeeting.hasOwnProperty("conferenceCall");

assert(hasOwnProperty, 'hasOwnProperty !== true');
assert(verifyTimeProp, 'verifyTimeProp !== true');
assert(verifyNameProp, 'verifyNameProp !== true');
assert(verifyCallProp, 'verifyCallProp !== true');
