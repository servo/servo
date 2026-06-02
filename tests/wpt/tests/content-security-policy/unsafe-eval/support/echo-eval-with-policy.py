def main(request, response):
    policy = request.GET.first(b"policy")
    return [(b"Content-Type", b"text/html"), (b"Content-Security-Policy", policy)], b"""
<!DOCTYPE html>
<html>
<script>
function check_eval(context) {
  context.eval_check_variable = 0;
  try {
    id = context.eval("eval_check_variable + 1");
  } catch (e) {
    if (e instanceof EvalError) {
      if (context.eval_check_variable === 0)
        return "blocked";
      else
        return "EvalError exception, but eval was executed";
    } else {
      return "Unexpected exception: " + e.message;
    }
  }
  return "allowed";
}

window.parent.postMessage({
  evalInIframe: check_eval(window),
  evalInParent: check_eval(parent),
});
</script>
</html>
"""
