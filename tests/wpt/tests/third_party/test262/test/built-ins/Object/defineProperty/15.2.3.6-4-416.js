// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-416
description: >
    ES5 Attributes - Inherited properties whose [[Enumerable]]
    attribute is set to false is non-enumerable (Object.create)
---*/

var appointment = {};

Object.defineProperty(appointment, "startTime", {
  value: 1001,
  writable: false,
  enumerable: false,
  configurable: true
});
Object.defineProperty(appointment, "name", {
  value: "NAME",
  writable: false,
  enumerable: false,
  configurable: true
});

var meeting = Object.create(appointment);
Object.defineProperty(meeting, "conferenceCall", {
  value: "In-person meeting",
  writable: false,
  enumerable: false,
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
assert.sameValue(verifyTimeProp, false, 'verifyTimeProp');
assert.sameValue(verifyNameProp, false, 'verifyNameProp');
assert.sameValue(verifyCallProp, false, 'verifyCallProp');
