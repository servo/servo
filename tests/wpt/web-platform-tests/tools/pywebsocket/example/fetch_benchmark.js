// Copyright 2015 Google Inc. All rights reserved.
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

var timerID = null;

function sendBenchmarkStep(size, config, isWarmUp) {
  timerID = null;
  benchmark.startTimeInMs = null;

  // Prepare data.
  var dataArray = [];
  for (var i = 0; i < config.numFetches; ++i) {
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

  // Start time measuring.
  benchmark.startTimeInMs = getTimeStamp();

  // Start fetch.
  var promises = [];
  for (var i = 0; i < config.numFetches; ++i) {
    var data = dataArray[i];
    var promise = fetch(config.prefixUrl + '_send',
                        {method: 'POST', body: data})
        .then(function (response) {
          if (response.status != 200) {
            config.addToLog('Failed (status=' + response.status + ')');
            return Promise.reject();
          }
          // Check and warn if proxy is enabled.
          if (response.headers.get('Via') !== null) {
            config.addToLog('WARNING: proxy seems enabled.');
          }
          if (config.verifyData) {
            return response.text()
                .then(function(text) {
                  if (!verifyAcknowledgement(config, text, size)) {
                    return Promise.reject();
                  }
                });
          }
        });
    promises.push(promise);
  }

  // Finish and report time measuring.
  Promise.all(promises)
      .then(function() {
        if (benchmark.startTimeInMs == null) {
          config.addToLog('startTimeInMs not set');
          return Promise.reject();
        }
        calculateAndLogResult(config, size, benchmark.startTimeInMs,
          size * config.numFetches, isWarmUp);
        runNextTask(config);
      })
      .catch(function(e) {
        config.addToLog("ERROR: " + e);
        config.notifyAbort();
      });
}

function receiveBenchmarkStep(size, config, isWarmUp) {
  timerID = null;
  benchmark.startTimeInMs = null;

  // Start time measuring.
  benchmark.startTimeInMs = getTimeStamp();

  // Start fetch.
  var promises = [];
  for (var i = 0; i < config.numFetches; ++i) {
    var request;
    if (config.methodAndCache === 'GET-NOCACHE') {
      request = new Request(config.prefixUrl + '_receive_getnocache?' + size,
                            {method: 'GET'});
    } else if (config.methodAndCache === 'GET-CACHE') {
      request = new Request(config.prefixUrl + '_receive_getcache?' + size,
                            {method: 'GET'});
    } else {
      request = new Request(config.prefixUrl + '_receive',
                            {method: 'POST', body: size + ' none'});
    }
    var promise = fetch(request)
        .then(function(response) {
          if (response.status != 200) {
            config.addToLog('Failed (status=' + this.status + ')');
            return Promise.reject();
          }
          // Check and warn if proxy is enabled.
          if (response.headers.get('Via') !== null) {
            config.addToLog('WARNING: proxy seems enabled.');
          }
          if (config.dataType === 'arraybuffer') {
            return response.arrayBuffer()
                .then(function(arrayBuffer) {
                  return [arrayBuffer.byteLength,
                          (!config.verifyData ||
                           verifyArrayBuffer(arrayBuffer, 0x61))];
                });
          } else if (config.dataType == 'blob') {
            return response.blob()
                .then(function(blob) {
                  return new Promise(function(resolve, reject) {
                      if (config.verifyData) {
                        verifyBlob(config, blob, 0x61,
                            function(receivedSize, verificationResult) {
                              resolve([receivedSize, verificationResult]);
                            });
                      } else {
                        resolve([blob.size, true]);
                      }
                    });
                });
          } else {
            return response.text()
                .then(function(text) {
                  return [text.length,
                          (!config.verifyData ||
                           text == repeatString('a', text.length))];
                });
          }
        })
        .then(function(receivedSizeAndVerificationResult) {
          var receivedSize = receivedSizeAndVerificationResult[0];
          var verificationResult = receivedSizeAndVerificationResult[1];
          if (receivedSize !== size) {
            config.addToLog('Expected ' + size +
                            'B but received ' + receivedSize + 'B');
            return Promise.reject();
          }
          if (!verificationResult) {
            config.addToLog('Response verification failed');
            return Promise.reject();
          }
        });
    promises.push(promise);
  }

  // Finish and report time measuring.
  Promise.all(promises)
      .then(function() {
        if (benchmark.startTimeInMs == null) {
          config.addToLog('startTimeInMs not set');
          return Promise.reject();
        }
        calculateAndLogResult(config, size, benchmark.startTimeInMs,
                              size * config.numFetches, isWarmUp);
        runNextTask(config);
      })
      .catch(function(e) {
        config.addToLog("ERROR: " + e);
        config.notifyAbort();
      });
}


function getConfigString(config) {
  return '(' + config.dataType +
    ', verifyData=' + config.verifyData +
    ', ' + (isWorker ? 'Worker' : 'Main') +
    ', numFetches=' + config.numFetches +
    ', numIterations=' + config.numIterations +
    ', numWarmUpIterations=' + config.numWarmUpIterations +
    ')';
}

function startBenchmark(config) {
  clearTimeout(timerID);

  runNextTask(config);
}

function batchBenchmark(originalConfig) {
  originalConfig.addToLog('Batch benchmark');

  tasks = [];
  clearAverageData();

  var dataTypes = ['text', 'blob', 'arraybuffer'];
  var stepFuncs = [sendBenchmarkStep, receiveBenchmarkStep];
  var names = ['Send', 'Receive'];
  for (var i = 0; i < stepFuncs.length; ++i) {
    for (var j = 0; j < dataTypes.length; ++j) {
      var config = cloneConfig(originalConfig);
      config.dataType = dataTypes[j];
      addTasks(config, stepFuncs[i]);
      addResultReportingTask(config,
          names[i] + ' benchmark ' + getConfigString(config));
    }
  }

  startBenchmark(config);
}

function cleanup() {
}
