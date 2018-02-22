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

function sendBenchmarkStep(size, config, isWarmUp) {
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
      config.notifyAbort();
      return;
    }

    if (config.verifyData &&
        !verifyAcknowledgement(config, this.response, size)) {
      destroyAllXHRs();
      config.notifyAbort();
      return;
    }

    totalReplied += size;

    if (totalReplied < totalSize) {
      return;
    }

    if (benchmark.startTimeInMs == null) {
      config.addToLog('startTimeInMs not set');
      destroyAllXHRs();
      config.notifyAbort();
      return;
    }

    // Check and warn if proxy is enabled.
    if (this.getResponseHeader('Via') !== null) {
      config.addToLog('WARNING: proxy seems enabled.');
    }

    calculateAndLogResult(config, size, benchmark.startTimeInMs, totalSize,
        isWarmUp);

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

function receiveBenchmarkStep(size, config, isWarmUp) {
  timerID = null;

  benchmark.startTimeInMs = null;
  var totalSize = 0;
  var totalReplied = 0;

  var checkResultAndContinue = function (bytesReceived, verificationResult) {
    if (!verificationResult) {
      config.addToLog('Response verification failed');
      destroyAllXHRs();
      config.notifyAbort();
      return;
    }

    totalReplied += bytesReceived;

    if (totalReplied < totalSize) {
      return;
    }

    if (benchmark.startTimeInMs == null) {
      config.addToLog('startTimeInMs not set');
      destroyAllXHRs();
      config.notifyAbort();
      return;
    }

    calculateAndLogResult(config, size, benchmark.startTimeInMs, totalSize,
        isWarmUp);

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
      config.notifyAbort();
      return;
    }

    // Check and warn if proxy is enabled.
    if (this.getResponseHeader('Via') !== null) {
      config.addToLog('WARNING: proxy seems enabled.');
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
      config.notifyAbort();
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
    if (config.methodAndCache === 'GET-NOCACHE') {
      xhr.open('GET', config.prefixUrl + '_receive_getnocache?' + size,
          config.async);
      xhr.responseType = config.dataType;
      xhr.send();
    } else if (config.methodAndCache === 'GET-CACHE') {
      xhr.open('GET', config.prefixUrl + '_receive_getcache?' + size,
          config.async);
      xhr.responseType = config.dataType;
      xhr.send();
    } else {
      xhr.open('POST', config.prefixUrl + '_receive', config.async);
      xhr.responseType = config.dataType;
      xhr.send(size + ' none');
    }
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

function cleanup() {
}
