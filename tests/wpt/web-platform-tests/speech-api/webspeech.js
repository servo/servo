var DELAY = 500;  // In milliseconds.
var TIMEOUT = 2000;  // In milliseconds.  Used for most tests.
if (typeof(TIMEOUT_OVERRIDE) != 'undefined') {
  TIMEOUT = TIMEOUT_OVERRIDE;
}
GLOBAL_TIMEOUT = TIMEOUT + 2000;  // In milliseconds.
setup({timeout: GLOBAL_TIMEOUT});
var onstarted = false;
var neverFireTest = async_test('Events that should not fire');
var onstartTest = async_test('onstart');
var reco = new SpeechRecognition();

reco.onstart = onstartTest.step_func(function(event) {
  assert_false(onstarted, 'onstart should only fire once.');
  onstarted = true;
  onstartTest.done();
  beginTest();
});

reco.onend = function() {
  neverFireTest.done();
  for (var i = 0; i < doneOnEndTestList.length; i++) {
    doneOnEndTestList[i].done();
  }
};

function neverFireEvent(name) {
  return neverFireTest.step_func(function(event) {
    assert_unreached(name + ' should not fire.');
  });
}

var doneOnEndTestList = [];  // List of all test objects to call done at onend.

// Tally calls to count() and test against min/max when test ends.
// A max value of 0 indicates no maximum.
function CountTest(name, min, max) {
  doneOnEndTestList.push(this);
  this.name = name;
  this.min = min;
  this.max = max;
  this.sum = 0;
  this.asyncTest = async_test(name);

  this.count = function(increment) { this.sum += increment; };

  this.test = function() { return this.asyncTest; };

  this.done = function() {
    var cTest = this;
    this.asyncTest.step(function() {
      notes.innerHTML += cTest.name + ' occurred ' + cTest.sum + ' times.<br>';
      if (cTest.min == cTest.max) {
        assert_true(cTest.sum == cTest.min, cTest.name + ' occurred ' +
          cTest.sum + ' times and should have occurred ' +
          cTest.min + ' times.');
      } else {
        assert_true(cTest.sum >= cTest.min, cTest.name + ' occurred ' +
            cTest.sum + ' times and should have occurred at least ' +
            cTest.min + ' times.');
        assert_true(cTest.max == 0 || cTest.sum <= cTest.max, cTest.name +
            ' occurred ' + cTest.sum +
            ' times and should have occurred at most ' + cTest.max + ' times.');
      }
      if (cTest.whenDone) {
        cTest.whenDone();
      }
    });
    this.asyncTest.done();
  };
}

// Test for proper cycling of startEvent followed by endEvent.
function CycleTest(name) {
  doneOnEndTestList.push(this);
  this.name = name;
  this.count = 0;  // Counts number of start / end cycles.
  this.started = false; // Tracks whether last event was a start or end event.
  this.test = async_test(name + ' start/stop');

  this.startEvent = function() {
    var cycle = this;
    return this.test.step_func(function(event) {
      assert_true(onstarted, cycle.name + 'start fired before onstart.');
      assert_false(cycle.started, cycle.name + 'start fired twice without ' +
                   cycle.name + 'stop.');
      cycle.started = true;
    });
  };

  this.endEvent = function() {
    var cycle = this;
    return this.test.step_func(function(event) {
      assert_true(cycle.started, cycle.name + 'end fired before ' +
                  cycle.name + 'start.');
      cycle.started = false;
      cycle.count += 1;
    });
  };

  this.done = function() {
    var cycle = this;
    this.test.step(function() {
      assert_false(cycle.started, cycle.name + 'start fired but not ' +
                   cycle.name + 'end.');
      assert_true(cycle.count > 0, cycle.name + 'start never fired.');
      notes.innerHTML += cycle.name + ' cycled ' + cycle.count + ' times.<br>';
    });
    this.test.done();
  };
}
