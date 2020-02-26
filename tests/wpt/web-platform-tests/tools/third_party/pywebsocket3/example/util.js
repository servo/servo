// Copyright 2013, Google Inc.
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


// Utilities for example applications (for both main and worker thread).

var results = {};

function getTimeStamp() {
  return Date.now();
}

function formatResultInKiB(size, timePerMessageInMs, stddevTimePerMessageInMs,
    speed, printSize) {
  if (printSize) {
    return (size / 1024) +
        '\t' + timePerMessageInMs.toFixed(3) +
        (stddevTimePerMessageInMs == -1 ?
            '' :
            '\t' + stddevTimePerMessageInMs.toFixed(3)) +
        '\t' + speed.toFixed(3);
  } else {
    return speed.toString();
  }
}

function clearAverageData() {
  results = {};
}

function reportAverageData(config) {
  config.addToSummary(
      'Size[KiB]\tAverage time[ms]\tStddev time[ms]\tSpeed[KB/s]');
  for (var size in results) {
    var averageTimePerMessageInMs = results[size].sum_t / results[size].n;
    var speed = calculateSpeedInKB(size, averageTimePerMessageInMs);
    // Calculate sample standard deviation
    var stddevTimePerMessageInMs = Math.sqrt(
        (results[size].sum_t2 / results[size].n -
            averageTimePerMessageInMs * averageTimePerMessageInMs) *
        results[size].n /
        (results[size].n - 1));
    config.addToSummary(formatResultInKiB(
        size, averageTimePerMessageInMs, stddevTimePerMessageInMs, speed,
        true));
  }
}

function calculateSpeedInKB(size, timeSpentInMs) {
  return Math.round(size / timeSpentInMs * 1000) / 1000;
}

function calculateAndLogResult(config, size, startTimeInMs, totalSize,
    isWarmUp) {
  var timeSpentInMs = getTimeStamp() - startTimeInMs;
  var speed = calculateSpeedInKB(totalSize, timeSpentInMs);
  var timePerMessageInMs = timeSpentInMs / (totalSize / size);
  if (!isWarmUp) {
    config.measureValue(timePerMessageInMs);
    if (!results[size]) {
      results[size] = {n: 0, sum_t: 0, sum_t2: 0};
    }
    results[size].n ++;
    results[size].sum_t += timePerMessageInMs;
    results[size].sum_t2 += timePerMessageInMs * timePerMessageInMs;
  }
  config.addToLog(formatResultInKiB(size, timePerMessageInMs, -1, speed,
      config.printSize));
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

function fillArrayBuffer(buffer, c) {
  var i;

  var u32Content = c * 0x01010101;

  var u32Blocks = Math.floor(buffer.byteLength / 4);
  var u32View = new Uint32Array(buffer, 0, u32Blocks);
  // length attribute is slow on Chrome. Don't use it for loop condition.
  for (i = 0; i < u32Blocks; ++i) {
    u32View[i] = u32Content;
  }

  // Fraction
  var u8Blocks = buffer.byteLength - u32Blocks * 4;
  var u8View = new Uint8Array(buffer, u32Blocks * 4, u8Blocks);
  for (i = 0; i < u8Blocks; ++i) {
    u8View[i] = c;
  }
}

function verifyArrayBuffer(buffer, expectedChar) {
  var i;

  var expectedU32Value = expectedChar * 0x01010101;

  var u32Blocks = Math.floor(buffer.byteLength / 4);
  var u32View = new Uint32Array(buffer, 0, u32Blocks);
  for (i = 0; i < u32Blocks; ++i) {
    if (u32View[i] != expectedU32Value) {
      return false;
    }
  }

  var u8Blocks = buffer.byteLength - u32Blocks * 4;
  var u8View = new Uint8Array(buffer, u32Blocks * 4, u8Blocks);
  for (i = 0; i < u8Blocks; ++i) {
    if (u8View[i] != expectedChar) {
      return false;
    }
  }

  return true;
}

function verifyBlob(config, blob, expectedChar, doneCallback) {
  var reader = new FileReader(blob);
  reader.onerror = function() {
    config.addToLog('FileReader Error: ' + reader.error.message);
    doneCallback(blob.size, false);
  }
  reader.onloadend = function() {
    var result = verifyArrayBuffer(reader.result, expectedChar);
    doneCallback(blob.size, result);
  }
  reader.readAsArrayBuffer(blob);
}

function verifyAcknowledgement(config, message, size) {
  if (typeof message != 'string') {
    config.addToLog('Invalid ack type: ' + typeof message);
    return false;
  }
  var parsedAck = parseInt(message);
  if (isNaN(parsedAck)) {
    config.addToLog('Invalid ack value: ' + message);
    return false;
  }
  if (parsedAck != size) {
    config.addToLog(
        'Expected ack for ' + size + 'B but received one for ' + parsedAck +
        'B');
    return false;
  }

  return true;
}

function cloneConfig(obj) {
  var newObj = {};
  for (key in obj) {
    newObj[key] = obj[key];
  }
  return newObj;
}

var tasks = [];

function runNextTask(config) {
  var task = tasks.shift();
  if (task == undefined) {
    config.addToLog('Finished');
    cleanup();
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
    var multiplierIndex = 0;
    for (var size = config.startSize;
         size <= config.stopThreshold;
         ++multiplierIndex) {
      var task = stepFunc.bind(
          null,
          size,
          config,
          i < config.numWarmUpIterations);
      tasks.push(task);
      var multiplier = config.multipliers[
        multiplierIndex % config.multipliers.length];
      if (multiplier <= 1) {
        config.addToLog('Invalid multiplier ' + multiplier);
        config.notifyAbort();
        throw new Error('Invalid multipler');
      }
      size = Math.ceil(size * multiplier);
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

function stop(config) {
  clearTimeout(timerID);
  timerID = null;
  tasks = [];
  config.addToLog('Stopped');
  cleanup();
}

var worker;

function initWorker(origin) {
  worker = new Worker(origin + '/benchmark.js');
}

function doAction(config, isWindowToWorker, action) {
  if (isWindowToWorker) {
    worker.onmessage = function(addToLog, addToSummary,
                                measureValue, notifyAbort, message) {
      if (message.data.type === 'addToLog')
        addToLog(message.data.data);
      else if (message.data.type === 'addToSummary')
        addToSummary(message.data.data);
      else if (message.data.type === 'measureValue')
        measureValue(message.data.data);
      else if (message.data.type === 'notifyAbort')
        notifyAbort();
    }.bind(undefined, config.addToLog, config.addToSummary,
           config.measureValue, config.notifyAbort);
    config.addToLog = undefined;
    config.addToSummary = undefined;
    config.measureValue = undefined;
    config.notifyAbort = undefined;
    worker.postMessage({type: action, config: config});
  } else {
    if (action === 'sendBenchmark')
      sendBenchmark(config);
    else if (action === 'receiveBenchmark')
      receiveBenchmark(config);
    else if (action === 'batchBenchmark')
      batchBenchmark(config);
    else if (action === 'stop')
      stop(config);
  }
}
