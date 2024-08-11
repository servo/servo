'use strict';

// Queues a zero ms timer that yields 3 times using `yieldParams`, then posts 2
// more 0 ms timers.
//
// Returns {tasks, ids} where `tasks` is an array of promises associated with
// the timers and `ids` is an array of task ids appended to by the scheduled
// tasks.
function postTestTasks() {
  const tasks = [];
  const ids = [];

  tasks.push(new Promise(resolve => {
    setTimeout(async () => {
      ids.push('t1');
      for (let i = 1; i < 4; i++) {
        await scheduler.yield();
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

promise_test(async t => {
  const {tasks, ids} = postTestTasks();
  await Promise.all(tasks);
  assert_equals(ids.join(), 't1,y1,y2,y3,t2,t3');
}, 'yield() with timer tasks (inherit signal)');
