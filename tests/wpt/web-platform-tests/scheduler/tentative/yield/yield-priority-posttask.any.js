'use strict';

// Posts a postTask task with `yieldyTaskParams` that yields 3 times using
// `yieldParams`, then posts 2 tasks of each priority, in descending order.
//
// Returns {tasks, ids} where `tasks` is an array of promises returned by
// postTask and `ids` is an array of task ids appended to by the scheduled
// tasks.
function postTestTasks(yieldyTaskParams, yieldParams) {
  const tasks = [];
  const ids = [];

  tasks.push(scheduler.postTask(async () => {
    ids.push('y0');
    for (let i = 1; i < 4; i++) {
      await scheduler.yield(yieldParams);
      ids.push('y' + i);
    }
  }, yieldyTaskParams));

  tasks.push(
      scheduler.postTask(() => {ids.push('ub1')}, {priority: 'user-blocking'}));
  tasks.push(
      scheduler.postTask(() => {ids.push('ub2')}, {priority: 'user-blocking'}));
  tasks.push(
      scheduler.postTask(() => {ids.push('uv1')}, {priority: 'user-visible'}));
  tasks.push(
      scheduler.postTask(() => {ids.push('uv2')}, {priority: 'user-visible'}));
  tasks.push(
      scheduler.postTask(() => {ids.push('bg1')}, {priority: 'background'}));
  tasks.push(
      scheduler.postTask(() => {ids.push('bg2')}, {priority: 'background'}));
  return {tasks, ids};
}

// Expected task orders for `postTestTasks` tasks.
const taskOrders = {
  'user-blocking': 'y0,y1,y2,y3,ub1,ub2,uv1,uv2,bg1,bg2',
  'user-visible': 'ub1,ub2,y0,y1,y2,y3,uv1,uv2,bg1,bg2',
  'background': 'ub1,ub2,uv1,uv2,y0,y1,y2,y3,bg1,bg2',
};

const priorityConfigs = [
  {options: {}, expected: taskOrders['user-visible']},
  {options: {priority: 'user-visible'}, expected: taskOrders['user-visible']},
  {options: {priority: 'user-blocking'}, expected: taskOrders['user-blocking']},
  {options: {priority: 'background'}, expected: taskOrders['background']},
];

const fixedPrioritySignals = {
  'user-blocking': (new TaskController({priority: 'user-blocking'})).signal,
  'user-visible': (new TaskController({priority: 'user-visible'})).signal,
  'background': (new TaskController({priority: 'background'})).signal,
};

const signalConfigs = [
  {
    options: {signal: fixedPrioritySignals['user-visible']},
    expected: taskOrders['user-visible']
  },
  {
    options: {signal: fixedPrioritySignals['user-blocking']},
    expected: taskOrders['user-blocking']
  },
  {
    options: {signal: fixedPrioritySignals['background']},
    expected: taskOrders['background']
  },
];

promise_test(async t => {
  for (const config of priorityConfigs) {
    const {tasks, ids} = postTestTasks(config.options, config.options);
    await Promise.all(tasks);
    assert_equals(ids.join(), config.expected);
  }
}, 'yield() with postTask tasks (priority option)');

promise_test(async t => {
  for (const config of signalConfigs) {
    const {tasks, ids} = postTestTasks(config.options, config.options);
    await Promise.all(tasks);
    assert_equals(ids.join(), config.expected);
  }
}, 'yield() with postTask tasks (signal option)');

promise_test(async t => {
  const expected = 'y0,ub1,ub2,uv1,uv2,y1,y2,y3,bg1,bg2';
  const {tasks, ids} = postTestTasks(
      {priority: 'user-blocking'}, {priority: 'background'});
  await Promise.all(tasks);
  assert_equals(ids.join(), expected);
}, 'yield() with different priority from task (priority)');

promise_test(async t => {
  const expected = 'y0,ub1,ub2,uv1,uv2,y1,y2,y3,bg1,bg2';
  const bgSignal = (new TaskController({priority: 'background'})).signal;
  const {tasks, ids} =
      postTestTasks({priority: 'user-blocking'}, {signal: bgSignal});
  await Promise.all(tasks);
  assert_equals(ids.join(), expected);
}, 'yield() with different priority from task (signal)');

promise_test(async t => {
  const bgSignal = (new TaskController({priority: 'background'})).signal;
  const {tasks, ids} = postTestTasks(
      {priority: 'user-blocking'},
      {signal: bgSignal, priority: 'user-blocking'});
  await Promise.all(tasks);
  assert_equals(ids.join(), taskOrders['user-blocking']);
}, 'yield() priority overrides signal');
