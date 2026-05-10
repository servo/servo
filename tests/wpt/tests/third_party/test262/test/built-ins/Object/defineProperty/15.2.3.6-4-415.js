// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-415
description: >
    ES5 Attributes - Failed to add properties to an object when the
    object's prototype has properties with the same name and
    [[Writable]] set to false (Object.create)
includes: [propertyHelper.js]
---*/

var appointment = new Object();

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

//teamMeeting.name = "Team Meeting";
verifyNotWritable(teamMeeting, "name", "noCheckOwnProp");

var dateObj = new Date("10/31/2010 08:00");
//teamMeeting.startTime = dateObj;
verifyNotWritable(teamMeeting, "startTime", "noCheckOwnProp");

//teamMeeting.conferenceCall = "4255551212";
verifyNotWritable(teamMeeting, "conferenceCall", "noCheckOwnProp");

assert(!teamMeeting.hasOwnProperty("name"));
assert(!teamMeeting.hasOwnProperty("startTime"));
assert(!teamMeeting.hasOwnProperty('conferenceCall'));

assert.sameValue(teamMeeting.name, "NAME");
assert.sameValue(teamMeeting.startTime, 1001);
assert.sameValue(teamMeeting.conferenceCall, "In-person meeting");
