// META: title=close event test

async_test(t => {
  const channel = new MessageChannel();
  channel.port1.start();
  channel.port1.onclose = t.step_func_done();
  channel.port1.dispatchEvent(new Event('close'));
}, 'Close event listener added with onclose must be called.');
