#!/usr/bin/python3

import os
import urllib.parse
import hashlib

doctmpl = """\
<!doctype html>
<title>WebVTT cue data parser test %s</title>
<link rel="help" href="https://w3c.github.io/webvtt/#cue-text-parsing-rules">
<style>video { display:none }</style>
<script src=/resources/testharness.js></script>
<script src=/resources/testharnessreport.js></script>
<script src=/html/syntax/parsing/template.js></script>
<script src=/html/syntax/parsing/common.js></script>
<script src=../common.js></script>
<div id=log></div>
<script>
runTests([
%s
]);
</script>
"""

testobj = "{name:'%s', input:'%s', expected:'%s'}"

def appendtest(tests, input, expected):
    tests.append(testobj % (hashlib.sha1(input.encode('UTF-8')).hexdigest(), urllib.parse.quote(input[:-1]),  urllib.parse.quote(expected[:-1])))

files = os.listdir('dat/')
for file in files:
    if os.path.isdir('dat/'+file) or file[0] == ".":
        continue

    tests = []
    input = ""
    expected = ""
    state = ""
    with open('dat/'+file, "r") as f:
        while True:
            line = f.readline()
            if not line:
                if state != "":
                    appendtest(tests, input, expected)
                    input = ""
                    expected = ""
                    state = ""
                break

            if line[0] == "#":
                state = line
                if line == "#document-fragment\n":
                    expected += bytes(line, 'UTF-8').decode('unicode-escape')
            elif state == "#data\n":
                input += bytes(line, 'UTF-8').decode('unicode-escape')
            elif state == "#errors\n":
                pass
            elif state == "#document-fragment\n":
                if line == "\n":
                    appendtest(tests, input, expected)
                    input = ""
                    expected = ""
                    state = ""
                else:
                    expected += bytes(line, 'UTF-8').decode('unicode-escape')
            else:
                raise Exception("failed to parse file %s:%s (state: %s)" % (file, line, state))

    name = os.path.splitext(file)[0]
    with open('tests/'+name+".html", "w") as out:
        out.write(doctmpl % (name, ",\n".join(tests)))
