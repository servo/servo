#!/usr/bin/python

import os
import urllib
import hashlib

doctmpl = """<!doctype html>
<title>WebVTT cue data parser test %s</title>
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
</script>"""

testobj = "{name:'%s', input:'%s', expected:'%s'}"

def appendtest(tests, input, expected):
    tests.append(testobj % (hashlib.sha1(input).hexdigest(), urllib.quote(input[:-1]),  urllib.quote(expected[:-1])))

files = os.listdir('dat/')
for file in files:
    if os.path.isdir('dat/'+file) or file[0] == ".":
        continue
    tests = []
    input = ""
    expected = ""
    state = ""
    f = open('dat/'+file)
    while 1:
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
                expected = expected + line
        elif state == "#data\n":
            input = input + line
        elif state == "#errors\n":
            pass
        elif state == "#document-fragment\n":
            if line == "\n":
                appendtest(tests, input, expected)
                input = ""
                expected = ""
                state = ""
            else:
                expected = expected + line
        else:
            raise Exception("failed to parse file "+file+" line:"+line+" (state: "+state+")")
    f.close()
    barename = file.replace(".dat", "")
    out = open('tests/'+barename+".html", "w")
    out.write(doctmpl % (barename, ",\n".join(tests)))
    out.close()
