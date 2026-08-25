// META: title=Scheduler: Tasks Run in Priority Order
// META: global=window,worker

promise_test(async t => {
  const runOrder = [];
  const schedule = (id, signal) => scheduler.postTask(() => { runOrder.push(id); }, {signal});

  const tasks = [];
  tasks.push(schedule('B1', TaskSignal.any([], {priority: 'background'})));
  tasks.push(schedule('B2', TaskSignal.any([], {priority: 'background'})));
  tasks.push(schedule('UV1', TaskSignal.any([], {priority: 'user-visible'})));
  tasks.push(schedule('UV2', TaskSignal.any([], {priority: 'user-visible'})));
  tasks.push(schedule('UB1', TaskSignal.any([], {priority: 'user-blocking'})));
  tasks.push(schedule('UB2', TaskSignal.any([], {priority: 'user-blocking'})));

  await Promise.all(tasks);

  assert_equals(runOrder.toString(),'UB1,UB2,UV1,UV2,B1,B2');
}, 'scheduler.postTask() tasks run in priority order with a fixed priority composite signal');

promise_test(async t => {
  const runOrder = [];
  const schedule = (id, priorityOrSignal) => {
    if (priorityOrSignal instanceof TaskSignal) {
      return scheduler.postTask(() => { runOrder.push(id); }, {signal: priorityOrSignal});
    } else {
      return scheduler.postTask(() => { runOrder.push(id); }, {priority: priorityOrSignal});
    }
  };

  const controller = new TaskController({priority: 'user-blocking'});
  const signal = TaskSignal.any([], {priority: controller.signal});

  const tasks = [];
  tasks.push(schedule('B1', signal));
  tasks.push(schedule('B2', signal));
  tasks.push(schedule('UV1', 'user-visible'));
  tasks.push(schedule('UV2', 'user-visible'));
  tasks.push(schedule('UB1', 'user-blocking'));
  tasks.push(schedule('UB2', 'user-blocking'));

  controller.setPriority('background');

  await Promise.all(tasks);

  assert_equals(runOrder.toString(),'UB1,UB2,UV1,UV2,B1,B2');
}, 'scheduler.postTask() tasks run in priority order with a dynamic priority composite signal');

promise_test(async t => {
  const runOrder = [];
  const schedule = (id, priorityOrSignal) => {
    if (priorityOrSignal instanceof TaskSignal) {
      return scheduler.postTask(() => { runOrder.push(id); }, {signal: priorityOrSignal});
    } else {
      return scheduler.postTask(() => { runOrder.push(id); }, {priority: priorityOrSignal});
    }
  };

  const parentSignal = TaskSignal.any([], {priority: 'background'});
  const signal = TaskSignal.any([], {priority: parentSignal});

  const tasks = [];
  tasks.push(schedule('B1', signal));
  tasks.push(schedule('B2', signal));
  tasks.push(schedule('UV1', 'user-visible'));
  tasks.push(schedule('UV2', 'user-visible'));
  tasks.push(schedule('UB1', 'user-blocking'));
  tasks.push(schedule('UB2', 'user-blocking'));

  await Promise.all(tasks);

  assert_equals(runOrder.toString(),'UB1,UB2,UV1,UV2,B1,B2');
}, 'scheduler.postTask() tasks run in priority order with a composite signal whose source has fixed priority');
