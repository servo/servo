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

  global.assert_xhr_event_order_matches = function(expected) {
    try {
      assert_array_equals(recorded_xhr_events, expected);
    } catch(e) {
      e.message += "\nRecorded events were:" + recorded_xhr_events.join(", ");
      e.message += "\nExpected events were:" + expected.join(", ");
      throw e;
    }
  }
}(this));
