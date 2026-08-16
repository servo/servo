'use strict';

if (self.importScripts) {
  importScripts('/resources/testharness.js');
}

setup({
  allow_uncaught_exception: true
});

// Deliberately more than a handful of promises: an implementation may track
// pending rejections differently once the list grows, so these cover that
// path in addition to the small-list one the other tests here exercise.
const COUNT = 20;

function rejectMany() {
  const reasons = [];
  const promises = [];
  for (let i = 0; i < COUNT; i++) {
    const reason = new Error('rejection ' + i);
    reasons.push(reason);
    promises.push(Promise.reject(reason));
  }
  return {promises, reasons};
}

// Resolves once an event of `type` has been seen for every promise in
// `expected`. Events for any other promise are ignored.
function waitForEvents(t, type, expected) {
  return new Promise(function(resolve) {
    const remaining = new Set(expected);
    const events = [];
    const listener = function(ev) {
      if (!remaining.has(ev.promise)) {
        return;
      }
      remaining.delete(ev.promise);
      events.push(ev);
      if (remaining.size === 0) {
        resolve(events);
      }
    };
    addEventListener(type, listener);
    t.add_cleanup(function() { removeEventListener(type, listener); });
  });
}

// Records every event of `type` fired for one of `promises`.
function recordEvents(t, type, promises) {
  const seen = [];
  const listener = function(ev) {
    if (promises.includes(ev.promise)) {
      seen.push(ev);
    }
  };
  addEventListener(type, listener);
  t.add_cleanup(function() { removeEventListener(type, listener); });
  return seen;
}

function tick(t) {
  return new Promise(function(resolve) { t.step_timeout(resolve, 50); });
}

promise_test(function(t) {
  const {promises, reasons} = rejectMany();

  return waitForEvents(t, 'unhandledrejection', promises).then(function(events) {
    assert_equals(events.length, COUNT);
    // The about-to-be-notified rejected promises list is iterated in order.
    assert_array_equals(events.map(function(ev) { return ev.promise; }),
                        promises, 'reported in rejection order');
    for (const ev of events) {
      assert_equals(ev.reason, reasons[promises.indexOf(ev.promise)]);
    }
  });
}, 'unhandledrejection: fires once for each of many rejections in one turn');

promise_test(function(t) {
  const {promises, reasons} = rejectMany();
  const seen = recordEvents(t, 'unhandledrejection', promises);

  const handled = promises.map(function(p, i) {
    return p.then(function() {
      assert_unreached('promise ' + i + ' should not fulfill');
    }, function(reason) {
      assert_equals(reason, reasons[i]);
    });
  });

  return Promise.all(handled).then(function() {
    return tick(t);
  }).then(function() {
    assert_equals(seen.length, 0, 'no unhandledrejection for handled promises');
  });
}, 'unhandledrejection: does not fire when many rejections are all handled in ' +
   'the same turn');

promise_test(function(t) {
  const {promises, reasons} = rejectMany();
  const expected = [];
  const shouldNotFire = [];

  promises.forEach(function(p, i) {
    if (i % 2 === 0) {
      shouldNotFire.push(p);
      p.catch(t.step_func(function(reason) {
        assert_equals(reason, reasons[i]);
      }));
    } else {
      expected.push(p);
    }
  });
  const seen = recordEvents(t, 'unhandledrejection', shouldNotFire);

  return waitForEvents(t, 'unhandledrejection', expected).then(function(events) {
    assert_equals(events.length, expected.length);
    assert_array_equals(events.map(function(ev) { return ev.promise; }),
                        expected, 'reported in rejection order');
    return tick(t);
  }).then(function() {
    assert_equals(seen.length, 0, 'no unhandledrejection for handled promises');
  });
}, 'unhandledrejection: fires only for the rejections left unhandled');

promise_test(function(t) {
  const {promises, reasons} = rejectMany();
  const seen = recordEvents(t, 'unhandledrejection', promises);

  // Handlers are attached back to front, so that an implementation tracking
  // positions in the list cannot rely on them being consumed in the order the
  // rejections were recorded.
  const handled = [];
  for (let i = COUNT - 1; i >= 0; i--) {
    const index = i;
    handled.push(promises[index].catch(function(reason) {
      assert_equals(reason, reasons[index]);
    }));
  }

  return Promise.all(handled).then(function() {
    return tick(t);
  }).then(function() {
    assert_equals(seen.length, 0, 'no unhandledrejection for handled promises');
  });
}, 'unhandledrejection: does not fire when many rejections are handled in ' +
   'reverse order');

promise_test(function(t) {
  const {promises, reasons} = rejectMany();

  return waitForEvents(t, 'unhandledrejection', promises).then(function() {
    return tick(t);
  }).then(function() {
    // The rejections have all been reported by now, so handling them at this
    // point has to produce rejectionhandled for every one of them.
    const handled = waitForEvents(t, 'rejectionhandled', promises);
    promises.forEach(function(p, i) {
      p.catch(t.step_func(function(reason) {
        assert_equals(reason, reasons[i]);
      }));
    });
    return handled;
  }).then(function(events) {
    assert_equals(events.length, COUNT);
    // One task is queued per promise as its handler is attached, and they all
    // go on the same task source, so they run in that order.
    assert_array_equals(events.map(function(ev) { return ev.promise; }),
                        promises, 'reported in the order they were handled');
  });
}, 'rejectionhandled: fires once for each of many rejections handled after ' +
   'they were reported');

promise_test(function(t) {
  const a = rejectMany();
  const seenA = recordEvents(t, 'unhandledrejection', a.promises);
  let b = null;

  const reported = new Promise(function(resolve) {
    const listener = t.step_func(function(ev) {
      if (b || !a.promises.includes(ev.promise)) {
        return;
      }
      // The list was cleared before this event was fired, so rejecting a
      // second batch now refills it from the start, giving those promises the
      // positions the first batch was recorded at. Handling the first batch
      // at this point must not disturb the second.
      b = rejectMany();
      const bReported = waitForEvents(t, 'unhandledrejection', b.promises);
      a.promises.forEach(function(p, i) {
        p.catch(t.step_func(function(reason) {
          assert_equals(reason, a.reasons[i]);
        }));
      });
      resolve(bReported);
    });
    addEventListener('unhandledrejection', listener);
    t.add_cleanup(function() {
      removeEventListener('unhandledrejection', listener);
    });
  });

  return reported.then(function(events) {
    assert_equals(events.length, COUNT);
    assert_array_equals(events.map(function(ev) { return ev.promise; }),
                        b.promises, 'reported in rejection order');
    // Only the first of the earlier batch is reported: the rest were handled
    // while that report was being delivered.
    assert_equals(seenA.length, 1);
  });
}, 'unhandledrejection: handling a reported batch does not suppress a batch ' +
   'rejected after it');

done();
