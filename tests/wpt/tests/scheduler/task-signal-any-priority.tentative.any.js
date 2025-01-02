// META: global=window,worker

test((t) => {
  const signal = TaskSignal.any([]);
  assert_true(signal instanceof TaskSignal);
  assert_equals(signal.priority, 'user-visible');
}, "TaskSignal.any() returns a user-visible TaskSignal when no priority is specified");

test((t) => {
  let signal = TaskSignal.any([], {priority: 'user-blocking'});
  assert_equals(signal.priority, 'user-blocking');

  signal = TaskSignal.any([], {priority: 'user-visible'});
  assert_equals(signal.priority, 'user-visible');

  signal = TaskSignal.any([], {priority: 'background'});
  assert_equals(signal.priority, 'background');
}, "TaskSignal.any() returns a signal with the correct priority when intialized with a string");

test((t) => {
  let controller = new TaskController({priority: 'user-blocking'});
  let signal = TaskSignal.any([], {priority: controller.signal});
  assert_equals(signal.priority, 'user-blocking');

  controller = new TaskController({priority: 'user-visible'});
  signal = TaskSignal.any([], {priority: controller.signal});
  assert_equals(signal.priority, 'user-visible');

  controller = new TaskController({priority: 'background'});
  signal = TaskSignal.any([], {priority: controller.signal});
  assert_equals(signal.priority, 'background');
}, "TaskSignal.any() returns a signal with the correct priority when intialized with a TaskSignal");

test((t) => {
  let controller = new TaskController({priority: 'user-blocking'});
  let signal = TaskSignal.any([], {priority: controller.signal});
  assert_equals(signal.priority, 'user-blocking');

  controller.setPriority('user-visible');
  assert_equals(signal.priority, 'user-visible');

  controller.setPriority('background');
  assert_equals(signal.priority, 'background');

  controller.setPriority('user-blocking');
  assert_equals(signal.priority, 'user-blocking');
}, "TaskSignal.any() returns a signal with dynamic priority");

test((t) => {
  const controller = new TaskController();
  const signal = TaskSignal.any([], {priority: controller.signal});

  let eventFiredCount = 0;
  signal.onprioritychange = t.step_func((e) => {
    assert_equals(e.target, signal,
        `The event target is the signal returned by TaskSignal.any()`);
    ++eventFiredCount;
  });

  controller.setPriority('background');
  assert_equals(eventFiredCount, 1);

  controller.setPriority('user-visible');
  assert_equals(eventFiredCount, 2);

  controller.setPriority('user-blocking');
  assert_equals(eventFiredCount, 3);
}, "Priority change events fire for composite signals");


test((t) => {
  const controller = new TaskController();
  let signal = TaskSignal.any([], {priority: controller.signal});
  signal = TaskSignal.any([], {priority: signal});
  signal = TaskSignal.any([], {priority: signal});
  signal = TaskSignal.any([], {priority: signal});
  signal = TaskSignal.any([], {priority: signal});

  assert_equals(signal.priority, 'user-visible');

  let eventFiredCount = 0;
  signal.onprioritychange = t.step_func((e) => {
    assert_equals(e.target, signal,
        "The event target is the signal returned by TaskSignal.any()");
    ++eventFiredCount;
  });

  controller.setPriority('background');
  assert_equals(eventFiredCount, 1);
  assert_equals(signal.priority, 'background');

  controller.setPriority('user-visible');
  assert_equals(eventFiredCount, 2);
  assert_equals(signal.priority, 'user-visible');

  controller.setPriority('user-blocking');
  assert_equals(eventFiredCount, 3);
  assert_equals(signal.priority, 'user-blocking');
}, "Priority change events fire for composite signals with intermediate sources");

test((t) => {
  const controller = new TaskController();
  const signals = [];
  const results = [];

  let id = 0;
  for (let i = 0; i < 3; i++) {
    const signal = TaskSignal.any([], {priority: controller.signal});
    const eventId = id++;
    signal.addEventListener('prioritychange', () => {
      results.push(eventId);
    });
    signals.push(signal);
  }
  for (let i = 0; i < 3; i++) {
    const signal = TaskSignal.any([], {priority: signals[i]});
    const eventId = id++;
    signal.addEventListener('prioritychange', () => {
      results.push(eventId);
    });
  }

  controller.setPriority('background');
  assert_equals(results.toString(), '0,1,2,3,4,5')

  controller.setPriority('user-blocking');
  assert_equals(results.toString(), '0,1,2,3,4,5,0,1,2,3,4,5')
}, "Priority change propagates to multiple dependent signals in the right order");

test((t) => {
  const controller = new TaskController();
  const signal = TaskSignal.any([], {priority: controller.signal});

  let fired = false;
  signal.onabort = t.step_func(() => {
    assert_unreached("The signal should not abort");
    fired = true;
  });

  controller.abort();
  assert_false(fired);
}, "TaskSignal.any() does not propagate abort when not given dependent abort signals");

test((t) => {
  const taskController = new TaskController();
  const abortController = new AbortController();
  const signal = TaskSignal.any([abortController.signal], {priority: taskController.signal});

  let priorityFireCount = 0;
  signal.onprioritychange = t.step_func(() => {
    ++priorityFireCount;
  });

  let abortFired = false;
  signal.onabort = t.step_func(() => {
    abortFired = true;
  });

  taskController.setPriority('background');
  assert_equals(signal.priority, 'background');
  assert_equals(priorityFireCount, 1);

  taskController.abort();
  assert_false(abortFired, "The signal should use abortController for abort");

  abortController.abort();
  assert_true(abortFired);

  taskController.setPriority('user-visible');
  assert_equals(signal.priority, 'user-visible');
  assert_equals(priorityFireCount, 2);
}, "TaskSignal.any() propagates abort and priority");


test((t) => {
  const controller = new TaskController();
  const signal = TaskSignal.any([AbortSignal.abort()], {priority: controller.signal});

  let fired = false;
  signal.onprioritychange = t.step_func(() => {
    fired = true;
  });

  controller.setPriority('background');
  assert_true(fired);
}, "TaskSignal.any() propagates priority after returning an aborted signal");

test((t) => {
  // Add a dependent in the initial event dispatch stage.
  let controller = new TaskController();
  let fired = false;
  controller.signal.onprioritychange = t.step_func(() => {
    fired = true;
    const newSignal = TaskSignal.any([], {priority: controller.signal});
    assert_equals(newSignal.priority, 'background');
    newSignal.onprioritychange = t.unreached_func('onprioritychange called');
  });
  controller.setPriority('background');
  assert_true(fired);

  // Add a dependent while signaling prioritychange on dependents.
  fired = false;
  controller = new TaskController();
  const signal = TaskSignal.any([], {priority: controller.signal});
  signal.onprioritychange = t.step_func(() => {
    fired = true;
    const newSignal = TaskSignal.any([], {priority: signal});
    assert_equals(newSignal.priority, 'background');
    newSignal.onprioritychange = t.unreached_func('onprioritychange called');
  });
  controller.setPriority('background');
  assert_true(fired);
}, "TaskSignal.any() does not fire prioritychange for dependents added during prioritychange");
