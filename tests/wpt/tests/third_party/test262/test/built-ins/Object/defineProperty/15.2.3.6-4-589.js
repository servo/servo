// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-589
description: >
    ES5 Attributes - Success to update value of property into of
    [[Proptotype]] internal property (Object.create)
---*/

var appointment = {};

var data1 = 1001;
Object.defineProperty(appointment, "startTime", {
  get: function() {
    return data1;
  },
  set: function(value) {
    data1 = value;
  },
  enumerable: true,
  configurable: true
});
var data2 = "NAME";
Object.defineProperty(appointment, "name", {
  get: function() {
    return data2;
  },
  set: function(value) {
    data2 = value;
  },
  enumerable: true,
  configurable: false
});

var meeting = Object.create(appointment);
var data3 = "In-person meeting";
Object.defineProperty(meeting, "conferenceCall", {
  get: function() {
    return data3;
  },
  set: function(value) {
    data3 = value;
  },
  enumerable: true,
  configurable: false
});

var teamMeeting = Object.create(meeting);
teamMeeting.name = "Team Meeting";
var dateObj = new Date("10/31/2010 08:00");
teamMeeting.startTime = dateObj;
teamMeeting.conferenceCall = "4255551212";

var hasOwnProperty = !teamMeeting.hasOwnProperty("name") &&
  !teamMeeting.hasOwnProperty("startTime") &&
  !teamMeeting.hasOwnProperty('conferenceCall');

assert(hasOwnProperty, 'hasOwnProperty !== true');
assert.sameValue(teamMeeting.name, "Team Meeting", 'teamMeeting.name');
assert.sameValue(teamMeeting.startTime, dateObj, 'teamMeeting.startTime');
assert.sameValue(teamMeeting.conferenceCall, "4255551212", 'teamMeeting.conferenceCall');
