function getGl() {
  var c = document.createElement("canvas");
  var gl = c.getContext("experimental-webgl");
  assert_true(!!gl, "Should be able to get a context.");
  return gl;
}

function shouldGenerateGLError(cx, glError, fn) {
  test(function() {
    fn();
    assert_equals(cx.getError(), glError);
  }, "Calling " + fn + " should generate a " + glError + " error.");
}
