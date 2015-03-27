// Copyright 2014 Google Inc. All rights reserved.
//
// Use of this source code is governed by a BSD-style
// license that can be found in the COPYING file or at
// https://developers.google.com/open-source/licenses/bsd

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

function sendBenchmarkStep(size, config) {
  timerID = null;

  var totalSize = 0;
  var totalReplied = 0;

  var onMessageHandler = function(event) {
    if (!verifyAcknowledgement(config, event.data, size)) {
      destroyAllSockets();
      return;
    }

    totalReplied += size;

    if (totalReplied < totalSize) {
      return;
    }

    calculateAndLogResult(config, size, benchmark.startTimeInMs, totalSize);

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

function receiveBenchmarkStep(size, config) {
  timerID = null;

  var totalSize = 0;
  var totalReplied = 0;

  var onMessageHandler = function(event) {
    var bytesReceived = event.data.byteLength;
    if (bytesReceived != size) {
      config.addToLog('Expected ' + size + 'B but received ' +
          bytesReceived + 'B');
      destroyAllSockets();
      return;
    }

    if (config.verifyData && !verifyArrayBuffer(event.data, 0x61)) {
      config.addToLog('Response verification failed');
      destroyAllSockets();
      return;
    }

    totalReplied += bytesReceived;

    if (totalReplied < totalSize) {
      return;
    }

    calculateAndLogResult(config, size, benchmark.startTimeInMs, totalSize);

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
  };
  return socket;
}

var tasks = [];

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

function runNextTask(config) {
  var task = tasks.shift();
  if (task == undefined) {
    config.addToLog('Finished');
    destroyAllSockets();
    return;
  }
  timerID = setTimeout(task, 0);
}

function buildLegendString(config) {
  var legend = ''
  if (config.printSize)
    legend = 'Message size in KiB, Time/message in ms, ';
  legend += 'Speed in kB/s';
  return legend;
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

function addTasks(config, stepFunc) {
  for (var i = 0;
      i < config.numWarmUpIterations + config.numIterations; ++i) {
    // Ignore the first |config.numWarmUpIterations| iterations.
    if (i == config.numWarmUpIterations)
      addResultClearingTask(config);

    var multiplierIndex = 0;
    for (var size = config.startSize;
         size <= config.stopThreshold;
         ++multiplierIndex) {
      var task = stepFunc.bind(
          null,
          size,
          config);
      tasks.push(task);
      size *= config.multipliers[
          multiplierIndex % config.multipliers.length];
    }
  }
}

function addResultReportingTask(config, title) {
  tasks.push(function(){
      timerID = null;
      config.addToSummary(title);
      reportAverageData(config);
      clearAverageData();
      runNextTask(config);
  });
}

function addResultClearingTask(config) {
  tasks.push(function(){
      timerID = null;
      clearAverageData();
      runNextTask(config);
  });
}

function sendBenchmark(config) {
  config.addToLog('Send benchmark');
  config.addToLog(buildLegendString(config));

  tasks = [];
  clearAverageData();
  addTasks(config, sendBenchmarkStep);
  addResultReportingTask(config, 'Send Benchmark ' + getConfigString(config));
  startBenchmark(config);
}

function receiveBenchmark(config) {
  config.addToLog('Receive benchmark');
  config.addToLog(buildLegendString(config));

  tasks = [];
  clearAverageData();
  addTasks(config, receiveBenchmarkStep);
  addResultReportingTask(config,
      'Receive Benchmark ' + getConfigString(config));
  startBenchmark(config);
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

function stop(config) {
  clearTimeout(timerID);
  timerID = null;
  config.addToLog('Stopped');
  destroyAllSockets();
}

onmessage = function (message) {
  var config = message.data.config;
  config.addToLog = workerAddToLog;
  config.addToSummary = workerAddToSummary;
  config.measureValue = workerMeasureValue;
  if (message.data.type === 'sendBenchmark')
    sendBenchmark(config);
  else if (message.data.type === 'receiveBenchmark')
    receiveBenchmark(config);
  else if (message.data.type === 'batchBenchmark')
    batchBenchmark(config);
  else if (message.data.type === 'stop')
    stop(config);
};
