test(function() {
  var testStyle = getComputedStyle(document.getElementById('test'));
  var refStyle = getComputedStyle(document.getElementById('ref'));
  for (var prop in testStyle) {
    assert_equals(testStyle[prop], refStyle[prop], prop);
  }
});
