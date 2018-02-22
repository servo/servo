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

function getConfigForPerformanceTest(connectionType, dataType, async,
                                     verifyData, numIterations,
                                     numWarmUpIterations) {
  var prefixUrl;
  if (connectionType === 'WebSocket') {
    prefixUrl = 'ws://' + location.host + '/benchmark_helper';
  } else {
    // XHR or fetch
    prefixUrl = 'http://' + location.host + '/073be001e10950692ccbf3a2ad21c245';
  }

  return {
    prefixUrl: prefixUrl,
    printSize: true,
    numXHRs: 1,
    numFetches: 1,
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
    initWorker(data.connectionType, 'http://' + location.host);
    action = data.benchmarkName;
  } else {
    action = 'stop';
  }

  var config = getConfigForPerformanceTest(data.connectionType, data.dataType,
                                           data.async, data.verifyData,
                                           data.numIterations,
                                           data.numWarmUpIterations);
  doAction(config, data.isWorker, action);
};
