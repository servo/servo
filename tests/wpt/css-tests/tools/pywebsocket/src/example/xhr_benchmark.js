// Copyright 2014 Google Inc. All rights reserved.
//
// Use of this source code is governed by a BSD-style
// license that can be found in the COPYING file or at
// https://developers.google.com/open-source/licenses/bsd

var isWorker = typeof importScripts !== "undefined";

if (isWorker) {
  // Running on a worker
  importScripts('util.js', 'util_worker.js');
}

// Namespace for holding globals.
var benchmark = {};
benchmark.startTimeInMs = 0;

var xhrs = [];

var timerID = null;

function destroyAllXHRs() {
  for (var i = 0; i < xhrs.length; ++i) {
    xhrs[i].onreadystatechange = null;
    // Abort XHRs if they are not yet DONE state.
    // Calling abort() here (i.e. in onreadystatechange handler) 
    // causes "NetworkError" messages in DevTools in sync mode,
    // even if it is after transition to DONE state.
    if (xhrs[i].readyState != XMLHttpRequest.DONE)
      xhrs[i].abort();
  }
  xhrs = [];
  // gc() might be needed for Chrome/Blob
}

function repeatString(str, count) {
  var data = '';
  var expChunk = str;
  var remain = count;
  while (true) {
    if (remain % 2) {
      data += expChunk;
      remain = (remain - 1) / 2;
    } else {
      remain /= 2;
    }

    if (remain == 0)
      break;

    expChunk = expChunk + expChunk;
  }
  return data;
}

function sendBenchmarkStep(size, config) {
  timerID = null;

  benchmark.startTimeInMs = null;
  var totalSize = 0;
  var totalReplied = 0;

  var onReadyStateChangeHandler = function () {
    if (this.readyState != this.DONE) {
      return;
    }

    if (this.status != 200) {
      config.addToLog('Failed (status=' + this.status + ')');
      destroyAllXHRs();
      return;
    }

    if (config.verifyData &&
        !verifyAcknowledgement(config, this.response, size)) {
      destroyAllXHRs();
      return;
    }

    totalReplied += size;

    if (totalReplied < totalSize) {
      return;
    }

    if (benchmark.startTimeInMs == null) {
      config.addToLog('startTimeInMs not set');
      destroyAllXHRs();
      return;
    }

    calculateAndLogResult(config, size, benchmark.startTimeInMs, totalSize);

    destroyAllXHRs();

    runNextTask(config);
  };

  for (var i = 0; i < config.numXHRs; ++i) {
    var xhr = new XMLHttpRequest();
    xhr.onreadystatechange = onReadyStateChangeHandler;
    xhrs.push(xhr);
  }

  var dataArray = [];

  for (var i = 0; i < xhrs.length; ++i) {
    var data = null;
    if (config.dataType == 'arraybuffer' ||
        config.dataType == 'blob') {
      data = new ArrayBuffer(size);

      fillArrayBuffer(data, 0x61);

      if (config.dataType == 'blob') {
        data = new Blob([data]);
      }
    } else {
      data = repeatString('a', size);
    }

    dataArray.push(data);
  }


  benchmark.startTimeInMs = getTimeStamp();
  totalSize = size * xhrs.length;

  for (var i = 0; i < xhrs.length; ++i) {
    var data = dataArray[i];
    var xhr = xhrs[i];
    xhr.open('POST', config.prefixUrl + '_send', config.async);
    xhr.send(data);
  }
}

function receiveBenchmarkStep(size, config) {
  timerID = null;

  benchmark.startTimeInMs = null;
  var totalSize = 0;
  var totalReplied = 0;

  var checkResultAndContinue = function (bytesReceived, verificationResult) {
    if (!verificationResult) {
      config.addToLog('Response verification failed');
      destroyAllXHRs();
      return;
    }

    totalReplied += bytesReceived;

    if (totalReplied < totalSize) {
      return;
    }

    if (benchmark.startTimeInMs == null) {
      config.addToLog('startTimeInMs not set');
      destroyAllXHRs();
      return;
    }

    calculateAndLogResult(config, size, benchmark.startTimeInMs, totalSize);

    destroyAllXHRs();

    runNextTask(config);
  }

  var onReadyStateChangeHandler = function () {
    if (this.readyState != this.DONE) {
      return;
    }

    if (this.status != 200) {
      config.addToLog('Failed (status=' + this.status + ')');
      destroyAllXHRs();
      return;
    }

    var bytesReceived = -1;
    if (this.responseType == 'arraybuffer') {
      bytesReceived = this.response.byteLength;
    } else if (this.responseType == 'blob') {
      bytesReceived = this.response.size;
    } else {
      bytesReceived = this.response.length;
    }
    if (bytesReceived != size) {
      config.addToLog('Expected ' + size +
          'B but received ' + bytesReceived + 'B');
      destroyAllXHRs();
      return;
    }

    if (this.responseType == 'arraybuffer') {
      checkResultAndContinue(bytesReceived,
          !config.verifyData || verifyArrayBuffer(this.response, 0x61));
    } else if (this.responseType == 'blob') {
      if (config.verifyData)
        verifyBlob(config, this.response, 0x61, checkResultAndContinue);
      else
        checkResultAndContinue(bytesReceived, true);
    } else {
      checkResultAndContinue(
          bytesReceived,
          !config.verifyData ||
              this.response == repeatString('a', this.response.length));
    }
  };

  for (var i = 0; i < config.numXHRs; ++i) {
    var xhr = new XMLHttpRequest();
    xhr.onreadystatechange = onReadyStateChangeHandler;
    xhrs.push(xhr);
  }

  benchmark.startTimeInMs = getTimeStamp();
  totalSize = size * xhrs.length;

  for (var i = 0; i < xhrs.length; ++i) {
    var xhr = xhrs[i];
    xhr.open('POST', config.prefixUrl + '_receive', config.async);
    xhr.responseType = config.dataType;
    xhr.send(size + ' none');
  }
}


function getConfigString(config) {
  return '(' + config.dataType +
    ', verifyData=' + config.verifyData +
    ', ' + (isWorker ? 'Worker' : 'Main') +
    ', ' + (config.async ? 'Async' : 'Sync') +
    ', numXHRs=' + config.numXHRs +
    ', numIterations=' + config.numIterations +
    ', numWarmUpIterations=' + config.numWarmUpIterations +
    ')';
}

function startBenchmark(config) {
  clearTimeout(timerID);
  destroyAllXHRs();

  runNextTask(config);
}

// TODO(hiroshige): the following code is the same as benchmark.html
// and some of them should be merged into e.g. util.js

var tasks = [];

function runNextTask(config) {
  var task = tasks.shift();
  if (task == undefined) {
    config.addToLog('Finished');
    destroyAllXHRs();
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

// --------------------------------

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

function batchBenchmark(originalConfig) {
  originalConfig.addToLog('Batch benchmark');

  tasks = [];
  clearAverageData();

  var dataTypes = ['text', 'blob', 'arraybuffer'];
  var stepFuncs = [sendBenchmarkStep, receiveBenchmarkStep];
  var names = ['Send', 'Receive'];
  var async = [true, false];
  for (var i = 0; i < stepFuncs.length; ++i) {
    for (var j = 0; j < dataTypes.length; ++j) {
      for (var k = 0; k < async.length; ++k) {
        var config = cloneConfig(originalConfig);
        config.dataType = dataTypes[j];
        config.async = async[k];

        // Receive && Non-Worker && Sync is not supported by the spec
        if (stepFuncs[i] === receiveBenchmarkStep && !isWorker &&
            !config.async)
          continue;

        addTasks(config, stepFuncs[i]);
        addResultReportingTask(config,
            names[i] + ' benchmark ' + getConfigString(config));
      }
    }
  }

  startBenchmark(config);
}


function stop(config) {
  destroyAllXHRs();
  clearTimeout(timerID);
  timerID = null;
  config.addToLog('Stopped');
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
