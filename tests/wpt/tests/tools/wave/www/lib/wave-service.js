function sendRequest(method, uri, headers, data, onSuccess, onError) {
  return sendRootRequest(
    method,
    WaveService.uriPrefix + uri,
    headers,
    data,
    onSuccess,
    onError
  );
}

function sendRootRequest(method, uri, headers, data, onSuccess, onError) {
  var xhr = new XMLHttpRequest();
  xhr.onload = function () {
    if (xhr.status === 200) {
      onSuccess(xhr.response);
    } else {
      if (onError) onError(xhr.status, xhr.response);
    }
  };
  xhr.onerror = function () {
    if (onError) onError();
  };
  xhr.open(method, uri, true);
  for (var header in headers) {
    xhr.setRequestHeader(header, headers[header]);
  }
  xhr.send(data);
  return xhr;
}

var WEB_ROOT = "{{WEB_ROOT}}";
var HTTP_PORT = "{{HTTP_PORT}}";
var HTTPS_PORT = "{{HTTPS_PORT}}";
var OPEN = "open";
var CLOSED = "closed";

var WaveService = {
  uriPrefix: WEB_ROOT,
  socket: {
    state: CLOSED,
    onMessage: function () {},
    onOpen: function () {},
    onClose: function () {},
    send: function () {},
    close: function () {},
    onStateChange: function () {},
  },
  // SESSIONS API
  createSession: function (configuration, onSuccess, onError) {
    var data = JSON.stringify({
      tests: configuration.tests,
      types: configuration.types,
      timeouts: configuration.timeouts,
      reference_tokens: configuration.referenceTokens,
      expiration_date: configuration.expirationDate,
      labels: configuration.labels,
    });
    sendRequest(
      "POST",
      "api/sessions",
      { "Content-Type": "application/json" },
      data,
      function (response) {
        var jsonObject = JSON.parse(response);
        onSuccess(jsonObject.token);
      },
      onError
    );
  },
  readSession: function (token, onSuccess, onError) {
    sendRequest(
      "GET",
      "api/sessions/" + token,
      null,
      null,
      function (response) {
        var jsonObject = JSON.parse(response);
        onSuccess({
          token: jsonObject.token,
          tests: jsonObject.tests,
          types: jsonObject.types,
          userAgent: jsonObject.user_agent,
          labels: jsonObject.labels,
          timeouts: jsonObject.timeouts,
          browser: jsonObject.browser,
          isPublic: jsonObject.is_public,
          referenceTokens: jsonObject.reference_tokens,
        });
      },
      onError
    );
  },
  readMultipleSessions: function (tokens, onSuccess, onError) {
    var requestsLeft = tokens.length;
    if (requestsLeft === 0) onSuccess([]);
    var configurations = [];
    for (var i = 0; i < tokens.length; i++) {
      var token = tokens[i];
      WaveService.readSession(
        token,
        function (configuration) {
          requestsLeft--;
          configurations.push(configuration);
          if (requestsLeft === 0) onSuccess(configurations);
        },
        function (status) {
          if (status === 404) requestsLeft--;
          if (status !== 404 && onError) onError();
          if (requestsLeft === 0) onSuccess(configurations);
        }
      );
    }
  },
  readSessionStatus: function (token, onSuccess, onError) {
    sendRequest(
      "GET",
      "api/sessions/" + token + "/status",
      null,
      null,
      function (response) {
        var jsonObject = JSON.parse(response);
        var dateStarted = null;
        if (jsonObject.date_started) {
          dateStarted = new Date(jsonObject.date_started);
        }
        var dateFinished = null;
        if (jsonObject.date_finished) {
          dateFinished = new Date(jsonObject.date_finished);
        }
        var expirationDate = null;
        if (jsonObject.expiration_date) {
          expirationDate = new Date(jsonObject.expiration_date);
        }
        onSuccess({
          token: jsonObject.token,
          dateStarted: dateStarted,
          dateFinished: dateFinished,
          testFilesCount: jsonObject.test_files_count,
          testFilesCompleted: jsonObject.test_files_completed,
          status: jsonObject.status,
          expirationDate: expirationDate,
        });
      },
      function () {
        if (onError) onError();
      }
    );
  },
  readMultipleSessionStatuses: function (tokens, onSuccess, onError) {
    var requestsLeft = tokens.length;
    if (requestsLeft === 0) onSuccess([]);
    var statuses = [];
    for (var i = 0; i < tokens.length; i++) {
      var token = tokens[i];
      WaveService.readSessionStatus(
        token,
        function (status) {
          requestsLeft--;
          statuses.push(status);
          if (requestsLeft === 0) onSuccess(statuses);
        },
        function () {
          requestsLeft--;
          if (requestsLeft === 0) onSuccess(statuses);
        }
      );
    }
  },
  readPublicSessions: function (onSuccess, onError) {
    sendRequest(
      "GET",
      "api/sessions/public",
      null,
      null,
      function (response) {
        var jsonObject = JSON.parse(response);
        onSuccess(jsonObject);
      },
      onError
    );
  },
  updateSession: function (token, configuration, onSuccess, onError) {
    var data = JSON.stringify({
      tests: configuration.tests,
      types: configuration.types,
      timeouts: configuration.timeouts,
      reference_tokens: configuration.referenceTokens,
      expiration_date: configuration.expirationDate,
      type: configuration.type,
    });
    sendRequest(
      "PUT",
      "api/sessions/" + token,
      { "Content-Type": "application/json" },
      data,
      function () {
        onSuccess();
      },
      onError
    );
  },
  updateLabels: function (token, labels, onSuccess, onError) {
    var data = JSON.stringify({ labels: labels });
    sendRequest(
      "PUT",
      "api/sessions/" + token + "/labels",
      { "Content-Type": "application/json" },
      data,
      function () {
        if (onSuccess) onSuccess();
      },
      onError
    );
  },
  findToken: function (fragment, onSuccess, onError) {
    sendRequest(
      "GET",
      "api/sessions/" + fragment,
      null,
      null,
      function (response) {
        var jsonObject = JSON.parse(response);
        onSuccess(jsonObject.token);
      },
      onError
    );
  },
  startSession: function (token, onSuccess, onError) {
    sendRequest(
      "POST",
      "api/sessions/" + token + "/start",
      null,
      null,
      function () {
        onSuccess();
      },
      onError
    );
  },
  pauseSession: function (token, onSuccess, onError) {
    sendRequest(
      "POST",
      "api/sessions/" + token + "/pause",
      null,
      null,
      function () {
        onSuccess();
      },
      onError
    );
  },
  stopSession: function (token, onSuccess, onError) {
    sendRequest(
      "POST",
      "api/sessions/" + token + "/stop",
      null,
      null,
      function () {
        onSuccess();
      },
      onError
    );
  },
  resumeSession: function (token, resumeToken, onSuccess, onError) {
    var data = JSON.stringify({ resume_token: resumeToken });
    sendRequest(
      "POST",
      "api/sessions/" + token + "/resume",
      { "Content-Type": "application/json" },
      data,
      function () {
        if (onSuccess) onSuccess();
      },
      function (response) {
        if (onError) onError(response);
      }
    );
  },
  deleteSession: function (token, onSuccess, onError) {
    sendRequest(
      "DELETE",
      "api/sessions/" + token,
      null,
      null,
      function () {
        onSuccess();
      },
      onError
    );
  },

  // TESTS API
  readTestList: function (onSuccess, onError) {
    sendRequest(
      "GET",
      "api/tests",
      null,
      null,
      function (response) {
        var jsonObject = JSON.parse(response);
        onSuccess(jsonObject);
      },
      onError
    );
  },
  readNextTest: function (token, onSuccess, onError) {
    sendRequest(
      "GET",
      "api/tests/" + token + "/next",
      null,
      null,
      function (response) {
        var jsonObject = JSON.parse(response);
        onSuccess(jsonObject.next_test);
      },
      onError
    );
  },
  readLastCompletedTests: function (token, resultTypes, onSuccess, onError) {
    var status = "";
    if (resultTypes) {
      for (var i = 0; i < resultTypes.length; i++) {
        var type = resultTypes[i];
        status += type + ",";
      }
    }
    sendRequest(
      "GET",
      "api/tests/" + token + "/last_completed?status=" + status,
      null,
      null,
      function (response) {
        var tests = JSON.parse(response);
        var parsedTests = [];
        for (var status in tests) {
          for (var i = 0; i < tests[status].length; i++) {
            var path = tests[status][i];
            parsedTests.push({ path: path, status: status });
          }
        }
        onSuccess(parsedTests);
      },
      onError
    );
  },
  readMalfunctioningTests: function (token, onSuccess, onError) {
    sendRequest(
      "GET",
      "api/tests/" + token + "/malfunctioning",
      null,
      null,
      function (response) {
        var tests = JSON.parse(response);
        onSuccess(tests);
      },
      function (response) {
        var errorMessage = JSON.parse(response).error;
        onError(errorMessage);
      }
    );
  },
  updateMalfunctioningTests: function (
    token,
    malfunctioningTests,
    onSuccess,
    onError
  ) {
    var data = JSON.stringify(malfunctioningTests);
    sendRequest(
      "PUT",
      "api/tests/" + token + "/malfunctioning",
      { "Content-Type": "application/json" },
      data,
      function () {
        onSuccess();
      },
      function (response) {
        var errorMessage = JSON.parse(response).error;
        onError(errorMessage);
      }
    );
  },
  readAvailableApis: function (onSuccess, onError) {
    sendRequest(
      "GET",
      "api/tests/apis",
      null,
      null,
      function (response) {
        var apis = JSON.parse(response);
        onSuccess(apis);
      },
      function (response) {
        if (!onError) return;
        var errorMessage = JSON.parse(response).error;
        onError(errorMessage);
      }
    );
  },
  readTestSubsets: function (onSuccess, onError) {
    sendRootRequest(
      "GET",
      "/test-subsets.json",
      null,
      null,
      function (response) {
        var subsets = JSON.parse(response).subsets;
        onSuccess(subsets);
      }
    );
  },

  // RESULTS API
  createResult: function (token, result, onSuccess, onError) {
    sendRequest(
      "POST",
      "api/results/" + token,
      { "Content-Type": "application/json" },
      JSON.stringify(result),
      function () {
        onSuccess();
      },
      onError
    );
  },
  readResults: function (token, onSuccess, onError) {
    sendRequest(
      "GET",
      "api/results/" + token,
      null,
      null,
      function (response) {
        onSuccess(JSON.parse(response));
      },
      onError
    );
  },
  readResultsCompact: function (token, onSuccess, onError) {
    sendRequest(
      "GET",
      "api/results/" + token + "/compact",
      null,
      null,
      function (response) {
        var jsonObject = JSON.parse(response);
        onSuccess(jsonObject);
      },
      onError
    );
  },
  readResultComparison: function (tokens, onSuccess, onError) {
    var comparison = {};
    var fetchComplete = function (results) {
      comparison.total = {};
      for (var i = 0; i < results.length; i++) {
        var result = results[i];
        var token = result.token;
        comparison[token] = {};
        for (var api in result) {
          if (api === "token") continue;
          comparison[token][api] = result[api].pass;
          if (!comparison.total[api]) {
            var total = 0;
            for (var status in result[api]) {
              total = total + result[api][status];
            }
            comparison.total[api] = total;
          }
        }
      }
      onSuccess(comparison);
    };
    var requestsLeft = tokens.length;
    if (requestsLeft === 0) onSuccess([]);
    var results = [];
    for (var i = 0; i < tokens.length; i++) {
      var token = tokens[i];
      (function (token) {
        WaveService.readResultsCompact(
          token,
          function (result) {
            requestsLeft--;
            result.token = token;
            results.push(result);
            if (requestsLeft === 0) fetchComplete(results);
          },
          function (responseStatus) {
            if (responseStatus === 404) requestsLeft--;
            if (status !== 404 && onError) onError();
            if (requestsLeft === 0) fetchComplete(results);
          }
        );
      })(token);
    }
  },
  downloadResults: function (token) {
    location.href = "api/results/" + token + "/export";
  },
  downloadApiResult: function (token, api) {
    location.href = "api/results/" + token + "/" + api + "/json";
  },
  downloadAllApiResults: function (token, api) {
    location.href = "api/results/" + token + "/json";
  },
  downloadReport: function (token, api) {
    location.href = "api/results/" + token + "/" + api + "/report";
  },
  importResults: function (data, onSuccess, onError) {
    sendRequest(
      "POST",
      "api/results/import",
      { "Content-Type": "application/octet-stream" },
      data,
      function (response) {
        var token = JSON.parse(response).token;
        onSuccess(token);
      },
      function (status, response) {
        var errorMessage;
        if (status === 500) {
          errorMessage = "Internal server error.";
        } else {
          errorMessage = JSON.parse(response).error;
        }
        onError(errorMessage);
      }
    );
  },
  readReportUri: function (token, api, onSuccess, onError) {
    sendRequest(
      "GET",
      "api/results/" + token + "/" + api + "/reporturl",
      null,
      null,
      function (response) {
        var jsonObject = JSON.parse(response);
        onSuccess(jsonObject.uri);
      },
      onError
    );
  },
  downloadMultiReport: function (tokens, api) {
    location.href = "api/results/" + api + "/report?tokens=" + tokens.join(",");
  },
  readMultiReportUri: function (tokens, api, onSuccess, onError) {
    sendRequest(
      "GET",
      "api/results/" + api + "/reporturl?tokens=" + tokens.join(","),
      null,
      null,
      function (response) {
        var jsonObject = JSON.parse(response);
        onSuccess(jsonObject.uri);
      },
      onError
    );
  },
  downloadResultsOverview: function (token) {
    location.href = "api/results/" + token + "/overview";
  },

  // DEVICES API
  _device_token: null,
  _deviceEventListeners: {},
  _deviceEventNumbers: {},
  registerDevice: function (onSuccess, onError) {
    sendRequest(
      "POST",
      "api/devices",
      null,
      null,
      function (response) {
        var data = JSON.parse(response);
        WaveService._device_token = data.token;
        onSuccess(data.token);
      },
      onError
    );
  },
  readDevice: function (token, onSuccess, onError) {
    sendRequest(
      "GET",
      "api/devices/" + token,
      null,
      null,
      function (response) {
        if (!onSuccess) return;
        var data = JSON.parse(response);
        onSuccess(data);
      },
      function (error) {
        if (!onError) return;
        onError(error);
      }
    );
  },
  DEVICE_ADDED_EVENT: "device_added",
  DEVICE_REMOVED_EVENT: "device_removed",
  START_SESSION: "start_session",
  addDeviceEventListener: function (token, callback) {
    var listeners = WaveService._deviceEventListeners;
    if (!listeners[token]) listeners[token] = [];
    listeners[token].push(callback);
    WaveService._deviceEventListeners = listeners;
    WaveService.listenDeviceEvents(token);
  },
  removeDeviceEventListener: function (callback) {
    var listeners = WaveService._deviceEventListeners;
    for (var token of Object.keys(listeners)) {
      var index = listeners[token].indexOf(callback);
      if (index === -1) continue;
      listeners[token].splice(index, 1);
      break;
    }
    WaveService._deviceEventListeners = listeners;
  },
  listenDeviceEvents: function (token) {
    var listeners = WaveService._deviceEventListeners;
    if (!listeners[token] || listeners.length === 0) return;
    var url = "api/devices/" + token + "/events";
    var lastEventNumber = WaveService._deviceEventNumbers[token];
    if (lastEventNumber) {
      url += "?last_active=" + lastEventNumber;
    }
    WaveService.listenHttpPolling(
      url,
      function (response) {
        if (!response) {
          WaveService.listenDeviceEvents(token);
          return;
        }
        for (var listener of listeners[token]) {
          listener(response);
        }
        WaveService._deviceEventNumbers[token] = lastEventNumber;
        WaveService.listenDeviceEvents(token);
      },
      function () {
        setTimeout(function () {
          WaveService.listenDeviceEvents();
        }, 1000);
      }
    );
  },
  sendDeviceEvent: function (device_token, event, onSuccess, onError) {
    var data = JSON.stringify({
      type: event.type,
      data: event.data,
    });
    sendRequest(
      "POST",
      "api/devices/" + device_token + "/events",
      { "Content-Type": "application/json" },
      data,
      onSuccess,
      onError
    );
  },
  addGlobalDeviceEventListener: function (callback) {
    WaveService._globalDeviceEventListeners.push(callback);
    WaveService.listenGlobalDeviceEvents();
  },
  removeGlobalDeviceEventListener: function (callback) {
    var index = WaveService._globalDeviceEventListeners.indexOf(callback);
    WaveService._globalDeviceEventListeners.splice(index, 1);
  },
  listenGlobalDeviceEvents: function () {
    var listeners = WaveService._globalDeviceEventListeners;
    if (listeners.length === 0) return;
    var query = "";
    if (WaveService._device_token) {
      query = "?device_token=" + WaveService._device_token;
    }
    WaveService.listenHttpPolling(
      "api/devices/events" + query,
      function (response) {
        if (!response) {
          WaveService.listenGlobalDeviceEvents();
          return;
        }
        for (var listener of listeners) {
          listener(response);
        }
        WaveService.listenGlobalDeviceEvents();
      },
      function () {
        setTimeout(function () {
          WaveService.listenGlobalDeviceEvents();
        }, 1000);
      }
    );
  },
  sendGlobalDeviceEvent: function (event, onSuccess, onError) {
    var data = JSON.stringify({
      type: event.type,
      data: event.data,
    });
    sendRequest(
      "POST",
      "api/devices/events",
      { "Content-Type": "application/json" },
      data,
      onSuccess,
      onError
    );
  },

  // GENERAL API
  readStatus: function (onSuccess, onError) {
    sendRequest(
      "GET",
      "api/status",
      null,
      null,
      function (response) {
        var data = JSON.parse(response);
        var configuration = {
          readSessionsEnabled: data.read_sessions_enabled,
          importResultsEnabled: data.import_results_enabled,
          reportsEnabled: data.reports_enabled,
          versionString: data.version_string,
          testTypeSelectionEnabled: data.test_type_selection_enabled,
          testFileSelectionEnabled: data.test_file_selection_enabled
        };
        onSuccess(configuration);
      },
      onError
    );
  },

  // UTILITY
  addRecentSession: function (token) {
    if (!token) return;
    var state = WaveService.getState();
    if (!state.recent_sessions) state.recent_sessions = [];
    if (state.recent_sessions.indexOf(token) !== -1) return;
    state.recent_sessions.unshift(token);
    WaveService.setState(state);
  },
  addRecentSessions: function (tokens) {
    for (var i = 0; i < tokens.length; i++) {
      var token = tokens[i];
      WaveService.addRecentSession(token);
    }
  },
  getPinnedSessions: function () {
    var state = WaveService.getState();
    if (!state || !state.pinned_sessions) return [];
    return state.pinned_sessions;
  },
  addPinnedSession: function (token) {
    if (!token) return;
    var state = WaveService.getState();
    if (!state.pinned_sessions) state.pinned_sessions = [];
    if (state.pinned_sessions.indexOf(token) !== -1) return;
    state.pinned_sessions.unshift(token);
    WaveService.setState(state);
  },
  getRecentSessions: function () {
    var state = WaveService.getState();
    if (!state || !state.recent_sessions) return [];
    return state.recent_sessions;
  },
  setRecentSessions: function (sessionTokens) {
    var state = WaveService.getState();
    state.recent_sessions = sessionTokens;
    WaveService.setState(state);
  },
  removePinnedSession: function (token) {
    if (!token) return;
    var state = WaveService.getState();
    if (!state.pinned_sessions) return;
    var index = state.pinned_sessions.indexOf(token);
    if (index === -1) return;
    state.pinned_sessions.splice(index, 1);
    WaveService.setState(state);
  },
  removeRecentSession: function (token) {
    var state = WaveService.getState();
    if (!state.recent_sessions) return;
    var index = state.recent_sessions.indexOf(token);
    if (index === -1) return;
    state.recent_sessions.splice(index, 1);
    WaveService.setState(state);
  },
  getState: function () {
    if (!window.localStorage) return null;
    var storage = window.localStorage;
    var state = JSON.parse(storage.getItem("wave"));
    if (!state) return {};
    return state;
  },
  setState: function (state) {
    if (!window.localStorage) return null;
    var storage = window.localStorage;
    storage.setItem("wave", JSON.stringify(state));
  },
  _globalDeviceEventListeners: [],
  _sessionEventListeners: {},
  _sessionEventNumbers: {},
  listenHttpPolling: function (url, onSuccess, onError) {
    var uniqueId = new Date().getTime();
    if (url.indexOf("?") === -1) {
      url = url + "?id=" + uniqueId;
    } else {
      url = url + "&id=" + uniqueId;
    }
    sendRequest(
      "GET",
      url,
      null,
      null,
      function (response) {
        if (!response) {
          onSuccess(null);
          return;
        }
        onSuccess(JSON.parse(response));
      },
      onError
    );
  },
  addSessionEventListener: function (token, callback) {
    var listeners = WaveService._sessionEventListeners;
    if (!listeners[token]) listeners[token] = [];
    if (listeners[token].indexOf(callback) >= 0) return;
    listeners[token].push(callback);
    WaveService._sessionEventListeners = listeners;
    WaveService.listenSessionEvents(token);
  },
  removeSessionEventListener: function (callback) {
    var listeners = WaveService._sessionEventListeners;
    for (var token of Object.keys(listeners)) {
      var index = listeners[token].indexOf(callback);
      if (index === -1) continue;
      listeners[token].splice(index, 1);
      break;
    }
    WaveService._sessionEventListeners = listeners;
  },
  listenSessionEvents: function (token) {
    var listeners = WaveService._sessionEventListeners;
    if (!listeners[token] || listeners.length === 0) return;
    var url = "api/sessions/" + token + "/events";
    var lastEventNumber = WaveService._sessionEventNumbers[token];
    if (lastEventNumber) {
      url += "?last_event=" + lastEventNumber;
    }
    WaveService.listenHttpPolling(
      url,
      function (response) {
        if (!response) {
          WaveService.listenSessionEvents(token);
          return;
        }
        var lastEventNumber = 0;
        for (var listener of listeners[token]) {
          for (var event of response) {
            if (event.number > lastEventNumber) {
              lastEventNumber = event.number;
            }
            listener(event);
          }
        }
        WaveService._sessionEventNumbers[token] = lastEventNumber;
        WaveService.listenSessionEvents(token);
      },
      function () {
        setTimeout(function () {
          WaveService.listenSessionEvents();
        }, 1000);
      }
    );
  },
  openSession: function (token) {
    location.href = "/results.html?token=" + token;
  },
};

if (!Object.keys)
  Object.keys = function (o) {
    if (o !== Object(o))
      throw new TypeError("Object.keys called on a non-object");
    var k = [],
      p;
    for (p in o) if (Object.prototype.hasOwnProperty.call(o, p)) k.push(p);
    return k;
  };
