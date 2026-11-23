# Serves the same page as /common/dispatcher/remote-executor.html, but only
# after sleeping, so that the status line and headers are held back too. The
# "trickle" pipe would not do here: it delays the body only, and a navigation
# can commit as soon as the headers arrive.
import time

def main(request, response):
    delay = float(request.GET.first(b"delay", b"4"))
    time.sleep(delay)
    return 200, [(b"Content-Type", b"text/html"),
                 (b"Cache-Control", b"no-store")], b"""<!DOCTYPE html>
<meta charset="utf-8">
<body>
<script src="/common/dispatcher/dispatcher.js"></script>
<script>
  const uuid = new URLSearchParams(location.search).get("uuid");
  new Executor(uuid);
</script>
"""
