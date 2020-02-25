// Copyright 2014, Google Inc.
// All rights reserved.
//
// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions are
// met:
//
//     * Redistributions of source code must retain the above copyright
// notice, this list of conditions and the following disclaimer.
//     * Redistributions in binary form must reproduce the above
// copyright notice, this list of conditions and the following disclaimer
// in the documentation and/or other materials provided with the
// distribution.
//     * Neither the name of Google Inc. nor the names of its
// contributors may be used to endorse or promote products derived from
// this software without specific prior written permission.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
// "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
// LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
// A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT
// OWNER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
// SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT
// LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE,
// DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY
// THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
// (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
// OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.


// Utilities for example applications (for the main thread only).

var logBox = null;
var queuedLog = '';

var summaryBox = null;

function queueLog(log) {
  queuedLog += log + '\n';
}

function addToLog(log) {
  logBox.value += queuedLog;
  queuedLog = '';
  logBox.value += log + '\n';
  logBox.scrollTop = 1000000;
}

function addToSummary(log) {
  summaryBox.value += log + '\n';
  summaryBox.scrollTop = 1000000;
}

// value: execution time in milliseconds.
// config.measureValue is intended to be used in Performance Tests.
// Do nothing here in non-PerformanceTest.
function measureValue(value) {
}

// config.notifyAbort is called when the benchmark failed and aborted, and
// intended to be used in Performance Tests.
// Do nothing here in non-PerformanceTest.
function notifyAbort() {
}

function getIntFromInput(id) {
  return parseInt(document.getElementById(id).value);
}

function getStringFromRadioBox(name) {
  var list = document.getElementById('benchmark_form')[name];
  for (var i = 0; i < list.length; ++i)
    if (list.item(i).checked)
      return list.item(i).value;
  return undefined;
}
function getBoolFromCheckBox(id) {
  return document.getElementById(id).checked;
}

function getIntArrayFromInput(id) {
  var strArray = document.getElementById(id).value.split(',');
  return strArray.map(function(str) { return parseInt(str, 10); });
}

function getFloatArrayFromInput(id) {
  var strArray = document.getElementById(id).value.split(',');
  return strArray.map(parseFloat);
}
