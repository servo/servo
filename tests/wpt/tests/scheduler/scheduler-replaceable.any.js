// META: title=Scheduler: scheduler should be replaceable
// META: global=window,worker
'use strict';

test(() => {
  class Scheduler {
    constructor() {
      scheduler = this;
    }
  }
  new Scheduler();
}, 'Tests replacing window.scheduler with a different object');
