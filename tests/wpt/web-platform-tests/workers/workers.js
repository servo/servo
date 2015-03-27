function listenForMessages(t, worker) {
  worker.addEventListener('message', t.step_func(function(e) {
    if (e.data === 'done') {
      t.done();
    }
    assert_unreached(e.data);
  }), false);
}
