'use strict';

function postInheritPriorityTestTask(config) {
  const ids = [];
  const task = scheduler.postTask(async () => {
    await new Promise(resolve => setTimeout(resolve));
    await fetch('/common/blank.html');
    await new Promise(resolve => setTimeout(resolve));
    const subtask = scheduler.postTask(() => { ids.push('subtask'); }, {priority: config.subTaskPriority});
    await scheduler.yield();
    ids.push('yield');
    await subtask;
  }, config.taskOptions);
  return {task, ids}
}

for (let priority of ['user-blocking', 'background']) {
  const expected = priority == 'user-blocking' ? 'yield,subtask' : 'subtask,yield';
  promise_test(async t => {
    const config = {
      taskOptions: {priority},
      subTaskPriority: 'user-blocking',
    };
    const {task, ids} = postInheritPriorityTestTask(config);
    await task;
    assert_equals(ids.join(), expected);
  }, `yield() inherits priority (string) across promises (${priority})`);

  promise_test(async t => {
    const signal = (new TaskController({priority})).signal;
    const config = {
      taskOptions: {signal},
      subTaskPriority: 'user-blocking',
    };
    const {task, ids} = postInheritPriorityTestTask(config);
    await task;
    assert_equals(ids.join(), expected);
  }, `yield() inherits priority (signal) across promises (${priority})`);
}

promise_test(async t => {
  const controller = new TaskController();
  const signal = controller.signal;
  return scheduler.postTask(async () => {
    await new Promise(resolve => setTimeout(resolve));
    await fetch('/common/blank.html');
    await new Promise(resolve => setTimeout(resolve));
    controller.abort();
    const p = scheduler.yield();
    await promise_rejects_dom(t, 'AbortError', p);
  }, {signal});
}, `yield() inherits abort across promises`);
