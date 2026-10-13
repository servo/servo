setup({explicit_done:true});

function check(p, iframe) {
  var current = p.firstElementChild;
  var ref_sizes = current.getAttribute('sizes');
  var expect = current.currentSrc;
  if (expect) {
    expect = expect.split('?')[0];
  }
  while (current = current.nextElementSibling) {
    test(function() {
      if (expect === '' || expect === null || expect === undefined) {
        assert_unreached('ref currentSrc was ' + format_value(expect));
      }
      var got = current.currentSrc;
      assert_greater_than(got.indexOf('?'), -1, 'expected a "?" in currentSrc');
      got = got.split('?')[0];
      assert_equals(got, expect);
    }, current.outerHTML + ' ref sizes=' + format_value(ref_sizes) + ' (' + iframe.getAttribute('data-desc') + ')');
  }
}

function runTests() {
  var iframe = document.querySelector('iframe');
  assert_true(!!iframe, 'iframe element must exist');
  [].forEach.call(iframe.contentDocument.querySelectorAll('p'), function(p) {
    check(p, iframe);
  });
  done();
}

if (document.readyState === 'complete') {
  runTests();
} else {
  window.addEventListener('load', runTests);
}
