// META: global=window,worker
// META: title=EventSource: constructor (invalid URL)

test(() => {
  assert_throws_dom('SyntaxError', () => { new EventSource("http://this is invalid/"); });
});

test(() => {
  assert_throws_dom('SyntaxError', () => {
    new EventSource('http://example.com:eighty/');
  });
}, 'EventSource: constructor (invalid port)');

done();
