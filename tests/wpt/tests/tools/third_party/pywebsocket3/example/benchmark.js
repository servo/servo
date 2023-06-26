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


if (typeof importScripts !== "undefined") {
  // Running on a worker
  importScripts('util.js', 'util_worker.js');
}

// Namespace for holding globals.
var benchmark = {startTimeInMs: 0};

var sockets = [];
var numEstablishedSockets = 0;

var timerID = null;

function destroySocket(socket) {
  socket.onopen = null;
  socket.onmessage = null;
  socket.onerror = null;
  socket.onclose = null;
  socket.close();
}

function destroyAllSockets() {
  for (var i = 0; i < sockets.length; ++i) {
    destroySocket(sockets[i]);
  }
  sockets = [];
}

function sendBenchmarkStep(size, config, isWarmUp) {
  timerID = null;

  var totalSize = 0;
  var totalReplied = 0;

  var onMessageHandler = function(event) {
    if (!verifyAcknowledgement(config, event.data, size)) {
      destroyAllSockets();
      config.notifyAbort();
      return;
    }

    totalReplied += size;

    if (totalReplied < totalSize) {
      return;
    }

    calculateAndLogResult(config, size, benchmark.startTimeInMs, totalSize,
        isWarmUp);

    runNextTask(config);
  };

  for (var i = 0; i < sockets.length; ++i) {
    var socket = sockets[i];
    socket.onmessage = onMessageHandler;
  }

  var dataArray = [];

  while (totalSize < config.minTotal) {
    var buffer = new ArrayBuffer(size);

    fillArrayBuffer(buffer, 0x61);

    dataArray.push(buffer);
    totalSize += size;
  }

  benchmark.startTimeInMs = getTimeStamp();

  totalSize = 0;

  var socketIndex = 0;
  var dataIndex = 0;
  while (totalSize < config.minTotal) {
    var command = ['send'];
    command.push(config.verifyData ? '1' : '0');
    sockets[socketIndex].send(command.join(' '));
    sockets[socketIndex].send(dataArray[dataIndex]);
    socketIndex = (socketIndex + 1) % sockets.length;

    totalSize += size;
    ++dataIndex;
  }
}

function receiveBenchmarkStep(size, config, isWarmUp) {
  timerID = null;

  var totalSize = 0;
  var totalReplied = 0;

  var onMessageHandler = function(event) {
    var bytesReceived = event.data.byteLength;
    if (bytesReceived != size) {
      config.addToLog('Expected ' + size + 'B but received ' +
          bytesReceived + 'B');
      destroyAllSockets();
      config.notifyAbort();
      return;
    }

    if (config.verifyData && !verifyArrayBuffer(event.data, 0x61)) {
      config.addToLog('Response verification failed');
      destroyAllSockets();
      config.notifyAbort();
      return;
    }

    totalReplied += bytesReceived;

    if (totalReplied < totalSize) {
      return;
    }

    calculateAndLogResult(config, size, benchmark.startTimeInMs, totalSize,
        isWarmUp);

    runNextTask(config);
  };

  for (var i = 0; i < sockets.length; ++i) {
    var socket = sockets[i];
    socket.binaryType = 'arraybuffer';
    socket.onmessage = onMessageHandler;
  }

  benchmark.startTimeInMs = getTimeStamp();

  var socketIndex = 0;
  while (totalSize < config.minTotal) {
    sockets[socketIndex].send('receive ' + size);
    socketIndex = (socketIndex + 1) % sockets.length;

    totalSize += size;
  }
}

function createSocket(config) {
  // TODO(tyoshino): Add TCP warm up.
  var url = config.prefixUrl;

  config.addToLog('Connect ' + url);

  var socket = new WebSocket(url);
  socket.onmessage = function(event) {
    config.addToLog('Unexpected message received. Aborting.');
  };
  socket.onerror = function() {
    config.addToLog('Error');
  };
  socket.onclose = function(event) {
    config.addToLog('Closed');
    config.notifyAbort();
  };
  return socket;
}

function startBenchmark(config) {
  clearTimeout(timerID);
  destroyAllSockets();

  numEstablishedSockets = 0;

  for (var i = 0; i < config.numSockets; ++i) {
    var socket = createSocket(config);
    socket.onopen = function() {
      config.addToLog('Opened');

      ++numEstablishedSockets;

      if (numEstablishedSockets == sockets.length) {
        runNextTask(config);
      }
    };
    sockets.push(socket);
  }
}

function getConfigString(config) {
  return '(WebSocket' +
    ', ' + (typeof importScripts !== "undefined" ? 'Worker' : 'Main') +
    ', numSockets=' + config.numSockets +
    ', numIterations=' + config.numIterations +
    ', verifyData=' + config.verifyData +
    ', minTotal=' + config.minTotal +
    ', numWarmUpIterations=' + config.numWarmUpIterations +
    ')';
}

function batchBenchmark(config) {
  config.addToLog('Batch benchmark');
  config.addToLog(buildLegendString(config));

  tasks = [];
  clearAverageData();
  addTasks(config, sendBenchmarkStep);
  addResultReportingTask(config, 'Send Benchmark ' + getConfigString(config));
  addTasks(config, receiveBenchmarkStep);
  addResultReportingTask(config, 'Receive Benchmark ' +
      getConfigString(config));
  startBenchmark(config);
}

function cleanup() {
  destroyAllSockets();
}
