function check(protocol, domain, port, done) {
  var url = protocol + '://' + domain + ':' + port + '/echo';
  var ws = new WebSocket(url);

  ws.addEventListener('error', function() {
    done(false);
  });

  ws.addEventListener('open', function() {
    done(true);
  });
}

async_test(function(t) {
  check('wss', '{{browser_host}}', {{ports[h2][0]}}, t.step_func(function(result) {
    assert_true(result);

    t.done();
  }));
}, 'WSS over h2');
