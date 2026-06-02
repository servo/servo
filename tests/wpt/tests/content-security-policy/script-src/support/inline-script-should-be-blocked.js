var t;
async_test(t => {
  self.t = t;
  const s = document.createElement('script');
  s.onerror = t.step_func(function() {
    assert_unreached('Script error event should not be fired.');
  });
  s.onload = t.step_func(function() {
    assert_unreached('Script load event should not be fired.');
  });
  s.innerText = 'self.t.assert_unreached("Script should not run.");'
  document.body.appendChild(s);
  setTimeout(() => t.done(), 2000);
});
