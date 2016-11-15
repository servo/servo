(function(global) {
  var recorded_xhr_events = [];

  function record_xhr_event(e) {
    var prefix = e.target instanceof XMLHttpRequestUpload ? "upload." : "";
    recorded_xhr_events.push((prefix || "") + e.type + "(" + e.loaded + "," + e.total + "," + e.lengthComputable + ")");
  }

  global.prepare_xhr_for_event_order_test = function(xhr) {
    xhr.addEventListener("readystatechange", function(e) {
      recorded_xhr_events.push(xhr.readyState);
    });
    var events = ["loadstart", "progress", "abort", "timeout", "error", "load", "loadend"];
    for(var i=0; i<events.length; ++i) {
      xhr.addEventListener(events[i], record_xhr_event);
    }
    if ("upload" in xhr) {
      for(var i=0; i<events.length; ++i) {
        xhr.upload.addEventListener(events[i], record_xhr_event);
      }
    }
  }

  function getNextEvent(arr) {
    var eventStr = arr.shift();

    // we can only handle strings, numbers (readystates) and undefined
    if (eventStr === undefined) {
      return event;
    }
    if (typeof eventStr !== "string") {
      if (Number.isInteger(eventStr)) {
        eventStr = "readystatechange(" + eventStr + ")";
      } else {
        throw "Test error: unexpected event type " + eventStr;
      }
    }

    // parse out the general type, loaded and total values
    var type = eventStr.type = eventStr.split("(")[0].split(".").pop();
    eventStr.mayFollowOptionalProgressEvents = type == "progress" ||
      type == "load" || type == "abort" || type == "error";
    var loadedAndTotal = eventStr.match(/\((\d)+,(\d)+/);
    if (loadedAndTotal) {
      eventStr.loaded = parseInt(loadedAndTotal[0]);
      eventStr.total = parseInt(loadedAndTotal[1]);
    }

    return eventStr;
  }

  global.assert_xhr_event_order_matches = function(expected) {
    var recorded = recorded_xhr_events;
    var lastRecordedLoaded = -1;

    while(expected.length && recorded.length) {
      var currentExpected = getNextEvent(expected),
          currentRecorded = getNextEvent(recorded);

      // skip to the last progress event if we've hit one
      while (recorded.length && currentRecorded.type == "progress") {
        assert_greater(currentRecorded.loaded, lastRecordedLoaded,
                       "progress event 'loaded' values must only increase");
        lastRecordedLoaded = currentRecorded.loaded;
        currentRecorded = getNextEvent(recorded);
      }
      if (currentRecorded.type == "loadstart") {
        lastRecordedLoaded = -1;
      }

      assert_equals(currentRecorded, currentExpected);
    }
    if (recorded.length) {
      throw "\nUnexpected extra events: " + recorded.join(", ");
    }
    if (expected.length) {
      throw "\nExpected more events: " + expected.join(", ");
    }
  }
}(this));
