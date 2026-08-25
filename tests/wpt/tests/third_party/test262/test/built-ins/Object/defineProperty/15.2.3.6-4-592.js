// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-592
description: >
    ES5 Attributes - Inherited property is non-enumerable
    (Object.create)
---*/

var appointment = {};

var data1 = 1001;
Object.defineProperty(appointment, "startTime", {
  get: function() {
    return data1;
  },
  enumerable: false,
  configurable: true
});
var data2 = "NAME";
Object.defineProperty(appointment, "name", {
  get: function() {
    return data2;
  },
  enumerable: false,
  configurable: false
});

var meeting = Object.create(appointment);
var data3 = "In-person meeting";
Object.defineProperty(meeting, "conferenceCall", {
  get: function() {
    return data3;
  },
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
  !teamMeeting.hasOwnProperty('conferenceCall');

assert(hasOwnProperty, 'hasOwnProperty !== true');
assert.sameValue(verifyTimeProp, false, 'verifyTimeProp');
assert.sameValue(verifyNameProp, false, 'verifyNameProp');
assert.sameValue(verifyCallProp, false, 'verifyCallProp');
