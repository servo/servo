def main(request, response):
    policy = request.GET.first(b"policy")
    return [(b"Content-Type", b"text/html"), (b"Content-Security-Policy", policy)], b"""
<!DOCTYPE html>
<html>
<script>
var id = 0;
try {
  id = eval("id + 1");
} catch (e) {}
window.parent.postMessage(id === 1 ? "eval allowed" : "eval blocked");
</script>
</html>
"""
