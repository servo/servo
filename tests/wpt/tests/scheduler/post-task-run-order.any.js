// META: title=Scheduler: Tasks Run in Priority Order
// META: global=window,worker

promise_test(async t => {
  const runOrder = [];
  const schedule = (id, priority) => scheduler.postTask(() => { runOrder.push(id); }, {priority});

  // Post tasks in reverse priority order and expect they are run from highest
  // to lowest priority.
  const tasks = [];
  tasks.push(schedule('B1', 'background'));
  tasks.push(schedule('B2', 'background'));
  tasks.push(schedule('UV1', 'user-visible'));
  tasks.push(schedule('UV2', 'user-visible'));
  tasks.push(schedule('UB1', 'user-blocking'));
  tasks.push(schedule('UB2', 'user-blocking'));

  await Promise.all(tasks);

  assert_equals(runOrder.toString(),'UB1,UB2,UV1,UV2,B1,B2');
}, 'Test scheduler.postTask task run in priority order');
