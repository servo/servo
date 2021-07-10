async_test(function() {
  var client = new XMLHttpRequest();
  client.open("GET", "resources/delay.py?ms=100", true);

  client.timeout = 150;
  client.ontimeout = this.step_func(assert_unreached);
  client.onloadend = () => this.done();

  client.send();

  const start = performance.now();
  while (performance.now() - start < 200) { }
}, "Long tasks should not trigger load timeout")

