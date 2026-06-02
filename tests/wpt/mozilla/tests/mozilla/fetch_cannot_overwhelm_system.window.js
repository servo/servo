// META: title=Ensure multiple fetch do not crash the browser.

async_test(function(t) {
  onload = t.step_func(function() {
    var step;
    var xhr
    var url = '/';
    t.step_timeout(t.step_func_done(), 10);
    for (step = 0; step < 5000; step++) {
      xhr = new XMLHttpRequest();
      xhr.open('GET', url, true);
      xhr.send();
    }
  });
});
