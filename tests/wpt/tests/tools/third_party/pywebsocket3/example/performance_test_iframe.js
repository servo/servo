// Copyright 2020, Google Inc.
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

function perfTestAddToLog(text) {
  parent.postMessage({'command': 'log', 'value': text}, '*');
}

function perfTestAddToSummary(text) {
}

function perfTestMeasureValue(value) {
  parent.postMessage({'command': 'measureValue', 'value': value}, '*');
}

function perfTestNotifyAbort() {
  parent.postMessage({'command': 'notifyAbort'}, '*');
}

function getConfigForPerformanceTest(dataType, async,
                                     verifyData, numIterations,
                                     numWarmUpIterations) {

  return {
    prefixUrl: 'ws://' + location.host + '/benchmark_helper',
    printSize: true,
    numSockets: 1,
    // + 1 is for a warmup iteration by the Telemetry framework.
    numIterations: numIterations + numWarmUpIterations + 1,
    numWarmUpIterations: numWarmUpIterations,
    minTotal: 10240000,
    startSize: 10240000,
    stopThreshold: 10240000,
    multipliers: [2],
    verifyData: verifyData,
    dataType: dataType,
    async: async,
    addToLog: perfTestAddToLog,
    addToSummary: perfTestAddToSummary,
    measureValue: perfTestMeasureValue,
    notifyAbort: perfTestNotifyAbort
  };
}

var data;
onmessage = function(message) {
  var action;
  if (message.data.command === 'start') {
    data = message.data;
    initWorker('http://' + location.host);
    action = data.benchmarkName;
  } else {
    action = 'stop';
  }

  var config = getConfigForPerformanceTest(data.dataType, data.async,
                                           data.verifyData,
                                           data.numIterations,
                                           data.numWarmUpIterations);
  doAction(config, data.isWorker, action);
};
