'use strict';

// Queues a zero ms timer that yields 3 times using `yieldParams`, then posts 2
// more 0 ms timers.
//
// Returns {tasks, ids} where `tasks` is an array of promises associated with
// the timers and `ids` is an array of task ids appended to by the scheduled
// tasks.
function postTestTasks(yieldParams) {
  const tasks = [];
  const ids = [];

  tasks.push(new Promise(resolve => {
    setTimeout(async () => {
      ids.push('t1');
      for (let i = 1; i < 4; i++) {
        await scheduler.yield(yieldParams);
        ids.push('y' + i);
      }
      resolve();
    });
  }));

  tasks.push(new Promise(resolve => {
    setTimeout(() => { ids.push('t2'); resolve(); });
  }));
  tasks.push(new Promise(resolve => {
    setTimeout(() => { ids.push('t3'); resolve(); });
  }));
  return {tasks, ids};
}

// Expected task orders for `postTestTasks` tasks.
const taskOrders = {
  'user-blocking': 't1,y1,y2,y3,t2,t3',
  'user-visible': 't1,y1,y2,y3,t2,t3',
  'background': 't1,t2,t3,y1,y2,y3',
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
    const {tasks, ids} = postTestTasks(config.options);
    await Promise.all(tasks);
    assert_equals(ids.join(), config.expected);
  }
}, 'yield() with timer tasks (priority option)');

promise_test(async t => {
  for (const config of signalConfigs) {
    const {tasks, ids} = postTestTasks(config.options);
    await Promise.all(tasks);
    assert_equals(ids.join(), config.expected);
  }
}, 'yield() with timer tasks (signal option)');

promise_test(async t => {
  const {tasks, ids} = postTestTasks({priority: 'inherit'});
  await Promise.all(tasks);
  assert_equals(ids.join(), taskOrders['user-visible']);
}, 'yield() with timer tasks (inherit priority)');

promise_test(async t => {
  const {tasks, ids} = postTestTasks({signal: 'inherit'});
  await Promise.all(tasks);
  assert_equals(ids.join(), taskOrders['user-visible']);
}, 'yield() with timer tasks (inherit signal)');
