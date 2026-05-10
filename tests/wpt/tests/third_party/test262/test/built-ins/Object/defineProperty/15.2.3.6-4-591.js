// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-591
description: >
    ES5 Attributes - Fail to update value of property of
    [[Proptotype]] internal property (Object.create)
includes: [propertyHelper.js]
---*/

var appointment = {};

var data1 = 1001;
Object.defineProperty(appointment, "startTime", {
  get: function() {
    return data1;
  },
  enumerable: false,
  configurable: false
});
var data2 = "NAME";
Object.defineProperty(appointment, "name", {
  get: function() {
    return data2;
  },
  enumerable: false,
  configurable: true
});

var meeting = Object.create(appointment);
var data3 = "In-person meeting";
Object.defineProperty(meeting, "conferenceCall", {
  get: function() {
    return data3;
  },
  enumerable: false,
  configurable: false
});

var teamMeeting = Object.create(meeting);

verifyNotWritable(teamMeeting, "name", "nocheck");
verifyNotWritable(teamMeeting, "startTime", "nocheck");
verifyNotWritable(teamMeeting, "conferenceCall", "nocheck");

try {
  teamMeeting.name = "IE Team Meeting";
} catch (e) {
  assert(e instanceof TypeError);
}

try {
  var dateObj = new Date("10/31/2010 08:00");
  teamMeeting.startTime = dateObj;
} catch (e) {
  assert(e instanceof TypeError);
}

try {
  teamMeeting.conferenceCall = "4255551212";
} catch (e) {
  assert(e instanceof TypeError);
}


assert(!teamMeeting.hasOwnProperty("name"));
assert(!teamMeeting.hasOwnProperty("startTime"));
assert(!teamMeeting.hasOwnProperty('conferenceCall'));

assert.sameValue(teamMeeting.name, "NAME");
assert.sameValue(teamMeeting.startTime, 1001);
assert.sameValue(teamMeeting.conferenceCall, "In-person meeting");
