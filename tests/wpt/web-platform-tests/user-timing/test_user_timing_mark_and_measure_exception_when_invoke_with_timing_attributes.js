importScripts("/resources/testharness.js");
importScripts("resources/webperftestharness.js");

test(function() {
  for (var i in timingAttributes) {
    performance.mark(timingAttributes[i]);
    performance.clearMarks(timingAttributes[i]);

    performance.measure(timingAttributes[i]);
    performance.clearMeasures(timingAttributes[i]);
  }
}, "performance.mark and performance.measure should not throw if used with timing attribute values in workers");

done();
