import time


def main(request, response):
    response.headers.set(b"Content-Type", b"text/html")
    response.status = 200
    response.write_status_headers()

    # Chunk 1: Setup target and start template with first child, then start interval observer
    response.writer.write_content(b"""<!DOCTYPE html>
<meta charset="utf-8">
<body>
<div id="target" marker="dest-marker">
  <?start name="dest-marker">Original Content<?end>
</div>

<script>
  window.observedLengths = [];
  const intervalId = setInterval(() => {
    const tpl = document.getElementById('test-template');
    if (tpl) {
      const elementNodes = Array.from(tpl.content.childNodes).filter(n => n.nodeType === Node.ELEMENT_NODE);
      window.observedLengths.push(elementNodes.length);
    }
  }, 10);
</script>

<template for="dest-marker" buffer id="test-template">
  <span id="child1">One</span>
""")

    # Yield and wait to allow the interval to fire while the template is still open
    time.sleep(0.5)

    # Chunk 2: Append second child, close template, clear interval and post result
    response.writer.write_content(b"""  <span id="child2">Two</span>
</template>

<script>
  clearInterval(intervalId);

  // Wait a microtask to ensure finalization completed
  setTimeout(() => {
    const target = document.getElementById('target');
    const child1_ok = target.querySelector('#child1') && target.querySelector('#child1').textContent === 'One';
    const child2_ok = target.querySelector('#child2') && target.querySelector('#child2').textContent === 'Two';
    const tpl_removed = document.getElementById('test-template') === null;
    const progress_ok = window.observedLengths.includes(1);

    let passed = child1_ok && child2_ok && tpl_removed && progress_ok;
    let message = "";
    if (!passed) {
      message = `child1: ${child1_ok}, child2: ${child2_ok}, tpl_removed: ${tpl_removed}, progress_ok (lengths includes 1): ${progress_ok}. Observed: ${JSON.stringify(window.observedLengths)}`;
    }

    window.parent.postMessage({
      type: 'test-result',
      passed: passed,
      message: message
    }, "*");
  }, 0);
</script>
</body>
""")
