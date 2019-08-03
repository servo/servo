onerror = function(a, b, c, d) {
  y(); // the error is "not handled"
}
function x() {
  y();
}
x();