'use strict';

if (self.importScripts) {
  importScripts('/resources/testharness.js');
}

setup({
  allow_uncaught_exception: true
});

//
// Straightforward unhandledrejection tests
//
async_test(function(t) {
  var e = new Error();
  var p;

  onUnhandledSucceed(t, e, function() { return p; });

  p = Promise.reject(e);
}, 'unhandledrejection: from Promise.reject');

async_test(function(t) {
  var e = new Error();
  var p;

  onUnhandledSucceed(t, e, function() { return p; });

  p = new Promise(function(_, reject) {
    reject(e);
  });
}, 'unhandledrejection: from a synchronous rejection in new Promise');

async_test(function(t) {
  var e = new Error();
  var p;

  onUnhandledSucceed(t, e, function() { return p; });

  p = new Promise(function(_, reject) {
    queueTask(function() {
      reject(e);
    });
  });
}, 'unhandledrejection: from a task-delayed rejection');

async_test(function(t) {
  var e = new Error();
  var p;

  onUnhandledSucceed(t, e, function() { return p; });

  p = new Promise(function(_, reject) {
    setTimeout(function() {
      reject(e);
    }, 1);
  });
}, 'unhandledrejection: from a setTimeout-delayed rejection');

async_test(function(t) {
  var e = new Error();
  var e2 = new Error();
  var promise2;

  onUnhandledSucceed(t, e2, function() { return promise2; });

  var unreached = t.unreached_func('promise should not be fulfilled');
  promise2 = Promise.reject(e).then(unreached, function(reason) {
    t.step(function() {
      assert_equals(reason, e);
    });
    throw e2;
  });
}, 'unhandledrejection: from a throw in a rejection handler chained off of Promise.reject');

async_test(function(t) {
  var e = new Error();
  var e2 = new Error();
  var promise2;

  onUnhandledSucceed(t, e2, function() { return promise2; });

  var unreached = t.unreached_func('promise should not be fulfilled');
  promise2 = new Promise(function(_, reject) {
    setTimeout(function() {
      reject(e);
    }, 1);
  }).then(unreached, function(reason) {
    t.step(function() {
      assert_equals(reason, e);
    });
    throw e2;
  });
}, 'unhandledrejection: from a throw in a rejection handler chained off of a setTimeout-delayed rejection');

async_test(function(t) {
  var e = new Error();
  var e2 = new Error();
  var promise2;

  onUnhandledSucceed(t, e2, function() { return promise2; });

  var promise = new Promise(function(_, reject) {
    setTimeout(function() {
      reject(e);
      mutationObserverMicrotask(function() {
        var unreached = t.unreached_func('promise should not be fulfilled');
        promise2 = promise.then(unreached, function(reason) {
          t.step(function() {
            assert_equals(reason, e);
          });
          throw e2;
        });
      });
    }, 1);
  });
}, 'unhandledrejection: from a throw in a rejection handler attached one microtask after a setTimeout-delayed rejection');

async_test(function(t) {
  var e = new Error();
  var p;

  onUnhandledSucceed(t, e, function() { return p; });

  p = Promise.resolve().then(function() {
    return Promise.reject(e);
  });
}, 'unhandledrejection: from returning a Promise.reject-created rejection in a fulfillment handler');

async_test(function(t) {
  var e = new Error();
  var p;

  onUnhandledSucceed(t, e, function() { return p; });

  p = Promise.resolve().then(function() {
    throw e;
  });
}, 'unhandledrejection: from a throw in a fulfillment handler');

async_test(function(t) {
  var e = new Error();
  var p;

  onUnhandledSucceed(t, e, function() { return p; });

  p = Promise.resolve().then(function() {
    return new Promise(function(_, reject) {
      setTimeout(function() {
        reject(e);
      }, 1);
    });
  });
}, 'unhandledrejection: from returning a setTimeout-delayed rejection in a fulfillment handler');

async_test(function(t) {
  var e = new Error();
  var p;

  onUnhandledSucceed(t, e, function() { return p; });

  p = Promise.all([Promise.reject(e)]);
}, 'unhandledrejection: from Promise.reject, indirected through Promise.all');

async_test(function(t) {
  var p;

  var unhandled = function(ev) {
    if (ev.promise === p) {
      t.step(function() {
        assert_equals(ev.reason.name, 'InvalidStateError');
        assert_equals(ev.promise, p);
      });
      t.done();
    }
  };
  addEventListener('unhandledrejection', unhandled);
  ensureCleanup(t, unhandled);

  p = createImageBitmap(new Blob());
}, 'unhandledrejection: from createImageBitmap which is UA triggered');

//
// Negative unhandledrejection/rejectionhandled tests with immediate attachment
//

async_test(function(t) {
  var e = new Error();
  var p;

  onUnhandledFail(t, function() { return p; });

  var unreached = t.unreached_func('promise should not be fulfilled');
  p = Promise.reject(e).then(unreached, function() {});
}, 'no unhandledrejection/rejectionhandled: rejection handler attached synchronously to a promise from Promise.reject');

async_test(function(t) {
  var e = new Error();
  var p;

  onUnhandledFail(t, function() { return p; });

  var unreached = t.unreached_func('promise should not be fulfilled');
  p = Promise.all([Promise.reject(e)]).then(unreached, function() {});
}, 'no unhandledrejection/rejectionhandled: rejection handler attached synchronously to a promise from ' +
   'Promise.reject, indirecting through Promise.all');

async_test(function(t) {
  var e = new Error();
  var p;

  onUnhandledFail(t, function() { return p; });

  var unreached = t.unreached_func('promise should not be fulfilled');
  p = new Promise(function(_, reject) {
    reject(e);
  }).then(unreached, function() {});
}, 'no unhandledrejection/rejectionhandled: rejection handler attached synchronously to a synchronously-rejected ' +
   'promise created with new Promise');

async_test(function(t) {
  var e = new Error();
  var p;

  onUnhandledFail(t, function() { return p; });

  var unreached = t.unreached_func('promise should not be fulfilled');
  p = Promise.resolve().then(function() {
    throw e;
  }).then(unreached, function(reason) {
    t.step(function() {
      assert_equals(reason, e);
    });
  });
}, 'no unhandledrejection/rejectionhandled: rejection handler attached synchronously to a promise created from ' +
   'throwing in a fulfillment handler');

async_test(function(t) {
  var e = new Error();
  var p;

  onUnhandledFail(t, function() { return p; });

  var unreached = t.unreached_func('promise should not be fulfilled');
  p = Promise.resolve().then(function() {
    return Promise.reject(e);
  }).then(unreached, function(reason) {
    t.step(function() {
      assert_equals(reason, e);
    });
  });
}, 'no unhandledrejection/rejectionhandled: rejection handler attached synchronously to a promise created from ' +
   'returning a Promise.reject-created promise in a fulfillment handler');

async_test(function(t) {
  var e = new Error();
  var p;

  onUnhandledFail(t, function() { return p; });

  var unreached = t.unreached_func('promise should not be fulfilled');
  p = Promise.resolve().then(function() {
    return new Promise(function(_, reject) {
      setTimeout(function() {
        reject(e);
      }, 1);
    });
  }).then(unreached, function(reason) {
    t.step(function() {
      assert_equals(reason, e);
    });
  });
}, 'no unhandledrejection/rejectionhandled: rejection handler attached synchronously to a promise created from ' +
   'returning a setTimeout-delayed rejection in a fulfillment handler');

async_test(function(t) {
  var e = new Error();
  var p;

  onUnhandledFail(t, function() { return p; });

  queueTask(function() {
    p = Promise.resolve().then(function() {
      return Promise.reject(e);
    })
    .catch(function() {});
  });
}, 'no unhandledrejection/rejectionhandled: all inside a queued task, a rejection handler attached synchronously to ' +
   'a promise created from returning a Promise.reject-created promise in a fulfillment handler');

async_test(function(t) {
  var p;

  onUnhandledFail(t, function() { return p; });

  var unreached = t.unreached_func('promise should not be fulfilled');
  p = createImageBitmap(new Blob()).then(unreached, function() {});
}, 'no unhandledrejection/rejectionhandled: rejection handler attached synchronously to a promise created from ' +
   'createImageBitmap');

//
// Negative unhandledrejection/rejectionhandled tests with microtask-delayed attachment
//

async_test(function(t) {
  var e = new Error();
  var p;

  onUnhandledFail(t, function() { return p; });

  p = Promise.reject(e);
  mutationObserverMicrotask(function() {
    var unreached = t.unreached_func('promise should not be fulfilled');
    p.then(unreached, function() {});
  });
}, 'delayed handling: a microtask delay before attaching a handler prevents both events (Promise.reject-created ' +
   'promise)');

async_test(function(t) {
  var e = new Error();
  var p;

  onUnhandledFail(t, function() { return p; });

  p = new Promise(function(_, reject) {
    reject(e);
  });
  mutationObserverMicrotask(function() {
    var unreached = t.unreached_func('promise should not be fulfilled');
    p.then(unreached, function() {});
  });
}, 'delayed handling: a microtask delay before attaching a handler prevents both events (immediately-rejected new ' +
   'Promise-created promise)');

async_test(function(t) {
  var e = new Error();
  var p1;
  var p2;

  onUnhandledFail(t, function() { return p1; });
  onUnhandledFail(t, function() { return p2; });

  p1 = new Promise(function(_, reject) {
    mutationObserverMicrotask(function() {
      reject(e);
    });
  });
  p2 = Promise.all([p1]);
  mutationObserverMicrotask(function() {
    var unreached = t.unreached_func('promise should not be fulfilled');
    p2.then(unreached, function() {});
  });
}, 'delayed handling: a microtask delay before attaching the handler, and before rejecting the promise, indirected ' +
   'through Promise.all');

//
// Negative unhandledrejection/rejectionhandled tests with nested-microtask-delayed attachment
//

async_test(function(t) {
  var e = new Error();
  var p;

  onUnhandledFail(t, function() { return p; });

  p = Promise.reject(e);
  mutationObserverMicrotask(function() {
    Promise.resolve().then(function() {
      mutationObserverMicrotask(function() {
        Promise.resolve().then(function() {
          p.catch(function() {});
        });
      });
    });
  });
}, 'microtask nesting: attaching a handler inside a combination of mutationObserverMicrotask + promise microtasks');

async_test(function(t) {
  var e = new Error();
  var p;

  onUnhandledFail(t, function() { return p; });

  queueTask(function() {
    p = Promise.reject(e);
    mutationObserverMicrotask(function() {
      Promise.resolve().then(function() {
        mutationObserverMicrotask(function() {
          Promise.resolve().then(function() {
            p.catch(function() {});
          });
        });
      });
    });
  });
}, 'microtask nesting: attaching a handler inside a combination of mutationObserverMicrotask + promise microtasks, ' +
   'all inside a queueTask');

async_test(function(t) {
  var e = new Error();
  var p;

  onUnhandledFail(t, function() { return p; });

  setTimeout(function() {
    p = Promise.reject(e);
    mutationObserverMicrotask(function() {
      Promise.resolve().then(function() {
        mutationObserverMicrotask(function() {
          Promise.resolve().then(function() {
            p.catch(function() {});
          });
        });
      });
    });
  }, 0);
}, 'microtask nesting: attaching a handler inside a combination of mutationObserverMicrotask + promise microtasks, ' +
   'all inside a setTimeout');

async_test(function(t) {
  var e = new Error();
  var p;

  onUnhandledFail(t, function() { return p; });

  p = Promise.reject(e);
  Promise.resolve().then(function() {
    mutationObserverMicrotask(function() {
      Promise.resolve().then(function() {
        mutationObserverMicrotask(function() {
          p.catch(function() {});
        });
      });
    });
  });
}, 'microtask nesting: attaching a handler inside a combination of promise microtasks + mutationObserverMicrotask');

async_test(function(t) {
  var e = new Error();
  var p;

  onUnhandledFail(t, function() { return p; });

  queueTask(function() {
    p = Promise.reject(e);
    Promise.resolve().then(function() {
      mutationObserverMicrotask(function() {
        Promise.resolve().then(function() {
          mutationObserverMicrotask(function() {
            p.catch(function() {});
          });
        });
      });
    });
  });
}, 'microtask nesting: attaching a handler inside a combination of promise microtasks + mutationObserverMicrotask, ' +
   'all inside a queueTask');

async_test(function(t) {
  var e = new Error();
  var p;

  onUnhandledFail(t, function() { return p; });

  setTimeout(function() {
    p = Promise.reject(e);
    Promise.resolve().then(function() {
      mutationObserverMicrotask(function() {
        Promise.resolve().then(function() {
          mutationObserverMicrotask(function() {
            p.catch(function() {});
          });
        });
      });
    });
  }, 0);
}, 'microtask nesting: attaching a handler inside a combination of promise microtasks + mutationObserverMicrotask, ' +
   'all inside a setTimeout');


// For workers, queueTask() involves posting tasks to other threads, so
// the following tests don't work there.

if ('document' in self) {
  //
  // Negative unhandledrejection/rejectionhandled tests with task-delayed attachment
  //

  async_test(function(t) {
    var e = new Error();
    var p;

    onUnhandledFail(t, function() { return p; });

    var _reject;
    p = new Promise(function(_, reject) {
      _reject = reject;
    });
    _reject(e);
    queueTask(function() {
      var unreached = t.unreached_func('promise should not be fulfilled');
      p.then(unreached, function() {});
    });
  }, 'delayed handling: a task delay before attaching a handler prevents unhandledrejection');

  async_test(function(t) {
    var e = new Error();
    var p;

    onUnhandledFail(t, function() { return p; });

    p = Promise.reject(e);
    queueTask(function() {
      Promise.resolve().then(function() {
        p.catch(function() {});
      });
    });
  }, 'delayed handling: queueTask after promise creation/rejection, plus promise microtasks, is not too late to ' +
     'attach a rejection handler');

  async_test(function(t) {
    var e = new Error();
    var p;

    onUnhandledFail(t, function() { return p; });

    queueTask(function() {
      Promise.resolve().then(function() {
        Promise.resolve().then(function() {
          Promise.resolve().then(function() {
            Promise.resolve().then(function() {
              p.catch(function() {});
            });
          });
        });
      });
    });
    p = Promise.reject(e);
  }, 'delayed handling: queueTask before promise creation/rejection, plus many promise microtasks, is not too ' +
     'late to attach a rejection handler');

  async_test(function(t) {
    var e = new Error();
    var p;

    onUnhandledFail(t, function() { return p; });

    p = Promise.reject(e);
    queueTask(function() {
      Promise.resolve().then(function() {
        Promise.resolve().then(function() {
          Promise.resolve().then(function() {
            Promise.resolve().then(function() {
              p.catch(function() {});
            });
          });
        });
      });
    });
  }, 'delayed handling: queueTask after promise creation/rejection, plus many promise microtasks, is not too ' +
     'late to attach a rejection handler');
}

//
// Positive unhandledrejection/rejectionhandled tests with delayed attachment
//

async_test(function(t) {
  var e = new Error();
  var p;

  onUnhandledSucceed(t, e, function() { return p; });

  var _reject;
  p = new Promise(function(_, reject) {
    _reject = reject;
  });
  _reject(e);
  queueTask(function() {
    queueTask(function() {
      var unreached = t.unreached_func('promise should not be fulfilled');
      p.then(unreached, function() {});
    });
  });
}, 'delayed handling: a nested-task delay before attaching a handler causes unhandledrejection');

async_test(function(t) {
  var e = new Error();
  var p;

  onUnhandledSucceed(t, e, function() { return p; });

  p = Promise.reject(e);
  queueTask(function() {
    queueTask(function() {
      Promise.resolve().then(function() {
        p.catch(function() {});
      });
    });
  });
}, 'delayed handling: a nested-queueTask after promise creation/rejection, plus promise microtasks, is too ' +
   'late to attach a rejection handler');

async_test(function(t) {
  var e = new Error();
  var p;

  onUnhandledSucceed(t, e, function() { return p; });

  queueTask(function() {
    queueTask(function() {
      Promise.resolve().then(function() {
        Promise.resolve().then(function() {
          Promise.resolve().then(function() {
            Promise.resolve().then(function() {
              p.catch(function() {});
            });
          });
        });
      });
    });
  });
  p = Promise.reject(e);
}, 'delayed handling: a nested-queueTask before promise creation/rejection, plus many promise microtasks, is ' +
   'too late to attach a rejection handler');

async_test(function(t) {
  var e = new Error();
  var p;

  onUnhandledSucceed(t, e, function() { return p; });

  p = Promise.reject(e);
  queueTask(function() {
    queueTask(function() {
      Promise.resolve().then(function() {
        Promise.resolve().then(function() {
          Promise.resolve().then(function() {
            Promise.resolve().then(function() {
              p.catch(function() {});
            });
          });
        });
      });
    });
  });
}, 'delayed handling: a nested-queueTask after promise creation/rejection, plus many promise microtasks, is ' +
   'too late to attach a rejection handler');

async_test(function(t) {
  var unhandledPromises = [];
  var unhandledReasons = [];
  var e = new Error();
  var p;

  var unhandled = function(ev) {
    if (ev.promise === p) {
      t.step(function() {
        unhandledPromises.push(ev.promise);
        unhandledReasons.push(ev.reason);
      });
    }
  };
  var handled = function(ev) {
    if (ev.promise === p) {
      t.step(function() {
        assert_array_equals(unhandledPromises, [p]);
        assert_array_equals(unhandledReasons, [e]);
        assert_equals(ev.promise, p);
        assert_equals(ev.reason, e);
      });
    }
  };
  addEventListener('unhandledrejection', unhandled);
  addEventListener('rejectionhandled', handled);
  ensureCleanup(t, unhandled, handled);

  p = new Promise(function() {
    throw e;
  });
  setTimeout(function() {
    var unreached = t.unreached_func('promise should not be fulfilled');
    p.then(unreached, function(reason) {
      assert_equals(reason, e);
      setTimeout(function() { t.done(); }, 10);
    });
  }, 10);
}, 'delayed handling: delaying handling by setTimeout(,10) will cause both events to fire');

async_test(function(t) {
  var unhandledPromises = [];
  var unhandledReasons = [];
  var p;

  var unhandled = function(ev) {
    if (ev.promise === p) {
      t.step(function() {
        unhandledPromises.push(ev.promise);
        unhandledReasons.push(ev.reason.name);
      });
    }
  };
  var handled = function(ev) {
    if (ev.promise === p) {
      t.step(function() {
        assert_array_equals(unhandledPromises, [p]);
        assert_array_equals(unhandledReasons, ['InvalidStateError']);
        assert_equals(ev.promise, p);
        assert_equals(ev.reason.name, 'InvalidStateError');
      });
    }
  };
  addEventListener('unhandledrejection', unhandled);
  addEventListener('rejectionhandled', handled);
  ensureCleanup(t, unhandled, handled);

  p = createImageBitmap(new Blob());
  setTimeout(function() {
    var unreached = t.unreached_func('promise should not be fulfilled');
    p.then(unreached, function(reason) {
      assert_equals(reason.name, 'InvalidStateError');
      setTimeout(function() { t.done(); }, 10);
    });
  }, 10);
}, 'delayed handling: delaying handling rejected promise created from createImageBitmap will cause both events to fire');

//
// Miscellaneous tests about integration with the rest of the platform
//

async_test(function(t) {
  var e = new Error();
  var l = function(ev) {
    var order = [];
    mutationObserverMicrotask(function() {
      order.push(1);
    });
    setTimeout(function() {
      order.push(2);
      t.step(function() {
        assert_array_equals(order, [1, 2]);
      });
      t.done();
    }, 1);
  };
  addEventListener('unhandledrejection', l);
  ensureCleanup(t, l);
  Promise.reject(e);
}, 'mutationObserverMicrotask vs. queueTask ordering is not disturbed inside unhandledrejection events');

// For workers, queueTask() involves posting tasks to other threads, so
// the following tests don't work there.

if ('document' in self) {

  // For the next two see https://github.com/domenic/unhandled-rejections-browser-spec/issues/2#issuecomment-121121695
  // and the following comments.

  async_test(function(t) {
    var sequenceOfEvents = [];

    addEventListener('unhandledrejection', l);
    ensureCleanup(t, l);

    var p1 = Promise.reject();
    var p2;
    queueTask(function() {
      p2 = Promise.reject();
      queueTask(function() {
        sequenceOfEvents.push('queueTask');
        checkSequence();
      });
    });

    function l(ev) {
      if (ev.promise === p1 || ev.promise === p2) {
        sequenceOfEvents.push(ev.promise);
        checkSequence();
      }
    }

    function checkSequence() {
      if (sequenceOfEvents.length === 3) {
        t.step(function() {
          assert_array_equals(sequenceOfEvents, [p1, 'queueTask', p2]);
        });
        t.done();
      }
    }
  }, 'queueTask ordering vs. the task queued for unhandled rejection notification (1)');

  async_test(function(t) {
    var sequenceOfEvents = [];

    addEventListener('unhandledrejection', l);
    ensureCleanup(t, l);

    var p2;
    queueTask(function() {
      p2 = Promise.reject();
      queueTask(function() {
        sequenceOfEvents.push('queueTask');
        checkSequence();
      });
    });

    function l(ev) {
      if (ev.promise == p2) {
        sequenceOfEvents.push(ev.promise);
        checkSequence();
      }
    }

    function checkSequence() {
      if (sequenceOfEvents.length === 2) {
        t.step(function() {
          assert_array_equals(sequenceOfEvents, ['queueTask', p2]);
        });
        t.done();
      }
    }
  }, 'queueTask ordering vs. the task queued for unhandled rejection notification (2)');

  async_test(function(t) {
    var sequenceOfEvents = [];


    addEventListener('unhandledrejection', unhandled);
    addEventListener('rejectionhandled', handled);
    ensureCleanup(t, unhandled, handled);

    var p = Promise.reject();

    function unhandled(ev) {
      if (ev.promise === p) {
        sequenceOfEvents.push('unhandled');
        checkSequence();
        setTimeout(function() {
          queueTask(function() {
            sequenceOfEvents.push('task before catch');
            checkSequence();
          });

          p.catch(function() {
            sequenceOfEvents.push('catch');
            checkSequence();
          });

          queueTask(function() {
            sequenceOfEvents.push('task after catch');
           checkSequence();
          });

          sequenceOfEvents.push('after catch');
          checkSequence();
        }, 10);
      }
    }

    function handled(ev) {
      if (ev.promise === p) {
        sequenceOfEvents.push('handled');
        checkSequence();
      }
    }

    function checkSequence() {
      if (sequenceOfEvents.length === 6) {
        t.step(function() {
          assert_array_equals(sequenceOfEvents,
            ['unhandled', 'after catch', 'catch', 'task before catch', 'handled', 'task after catch']);
        });
        t.done();
      }
    }
  }, 'rejectionhandled is dispatched from a queued task, and not immediately');
}

//
// HELPERS
//

// This function queues a task in "DOM manipulation task source" in window
// context, but not in workers.
function queueTask(f) {
  if ('document' in self) {
    var d = document.createElement("details");
    d.ontoggle = function() {
      f();
    };
    d.setAttribute("open", "");
  } else {
    // We need to fix this to use something that can queue tasks in
    // "DOM manipulation task source" to ensure the order is correct
    var channel = new MessageChannel();
    channel.port1.onmessage = function() { channel.port1.close(); f(); };
    channel.port2.postMessage('abusingpostmessageforfunandprofit');
    channel.port2.close();
  }
}

function mutationObserverMicrotask(f) {
  if ('document' in self) {
    var observer = new MutationObserver(function() { f(); });
    var node = document.createTextNode('');
    observer.observe(node, { characterData: true });
    node.data = 'foo';
  } else {
    // We don't have mutation observers on workers, so just post a promise-based
    // microtask.
    Promise.resolve().then(function() { f(); });
  }
}

function onUnhandledSucceed(t, expectedReason, expectedPromiseGetter) {
  var l = function(ev) {
    if (ev.promise === expectedPromiseGetter()) {
      t.step(function() {
        assert_equals(ev.reason, expectedReason);
        assert_equals(ev.promise, expectedPromiseGetter());
      });
      t.done();
    }
  };
  addEventListener('unhandledrejection', l);
  ensureCleanup(t, l);
}

function onUnhandledFail(t, expectedPromiseGetter) {
  var unhandled = function(evt) {
    if (evt.promise === expectedPromiseGetter()) {
      t.step(function() {
        assert_unreached('unhandledrejection event is not supposed to be triggered');
      });
    }
  };
  var handled = function(evt) {
    if (evt.promise === expectedPromiseGetter()) {
      t.step(function() {
        assert_unreached('rejectionhandled event is not supposed to be triggered');
      });
    }
  };
  addEventListener('unhandledrejection', unhandled);
  addEventListener('rejectionhandled', handled);
  ensureCleanup(t, unhandled, handled);
  setTimeout(function() {
    t.done();
  }, 10);
}

function ensureCleanup(t, unhandled, handled) {
  t.add_cleanup(function() {
    if (unhandled)
      removeEventListener('unhandledrejection', unhandled);
    if (handled)
      removeEventListener('rejectionhandled', handled);
  });
}

done();
