# script to generate the generateKey tests

import os

here = os.path.dirname(__file__)

successes_html = """<!DOCTYPE html>
<meta charset=utf-8>
<meta name="timeout" content="long">
<title>WebCryptoAPI: generateKey() Successful Calls</title>
<link rel="author" title="Charles Engelke" href="mailto:w3c@engelke.com">
<link rel="help" href="https://www.w3.org/TR/WebCryptoAPI/#dfn-SubtleCrypto-method-generateKey">
<script src="/resources/testharness.js"></script>
<script src="/resources/testharnessreport.js"></script>

<script src="/WebCryptoAPI/util/helpers.js"></script>
<script src="successes.js"></script>

<h1>generateKey Tests for Good Parameters</h1>
<p>
    <strong>Warning!</strong> RSA key generation is intrinsically
    very slow, so the related tests can take up to
    several minutes to complete, depending on browser!
</p>

<div id="log"></div>
<script>
run_test([%s]);
</script>"""

failures_html = """<!DOCTYPE html>
<meta charset=utf-8>
<meta name="timeout" content="long">
<title>WebCryptoAPI: generateKey() for Failures</title>
<link rel="author" title="Charles Engelke" href="mailto:w3c@engelke.com">
<link rel="help" href="https://www.w3.org/TR/WebCryptoAPI/#dfn-SubtleCrypto-method-generateKey">
<script src="/resources/testharness.js"></script>
<script src="/resources/testharnessreport.js"></script>

<script src="/WebCryptoAPI/util/helpers.js"></script>
<script src="failures.js"></script>

<h1>generateKey Tests for Bad Parameters</h1>

<div id="log"></div>
<script>
run_test([%s]);
</script>
"""

successes_worker = """// META: timeout=long
importScripts("/resources/testharness.js");
importScripts("../util/helpers.js");
importScripts("successes.js");

run_test([%s]);
done();"""

failures_worker = """// META: timeout=long
importScripts("/resources/testharness.js");
importScripts("../util/helpers.js");
importScripts("failures.js");
run_test([%s]);
done();"""

names = ["AES-CTR", "AES-CBC", "AES-GCM", "AES-KW", "HMAC", "RSASSA-PKCS1-v1_5",
         "RSA-PSS", "RSA-OAEP", "ECDSA", "ECDH"]

for filename_pattern, template in [("test_successes_%s.html", successes_html),
                                   ("test_failures_%s.html", failures_html),
                                   ("successes_%s.worker.js", successes_worker),
                                   ("failures_%s.worker.js", failures_worker)]:
    for name in names:
        path = os.path.join(here, os.pardir, "generateKey", filename_pattern % name)
        with open(path, "w") as f:
            f.write(template % '"%s"' % name)
