#!/usr/bin/env python3

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.

import gzip
import json
import os
import re
import sys
import urllib.request
from html import escape as html_escape


TASKCLUSTER_ROOT_URL = "https://community-tc.services.mozilla.com"


def fetch(url):
    url = TASKCLUSTER_ROOT_URL + "/api/" + url
    print("Fetching " + url)
    response = urllib.request.urlopen(url)
    assert response.getcode() == 200
    encoding = response.info().get("Content-Encoding")
    if not encoding:
        return response
    elif encoding == "gzip":
        return gzip.GzipFile(fileobj=response)
    else:
        raise ValueError("Unsupported Content-Encoding: %s" % encoding)


def fetch_json(url):
    with fetch(url) as response:
        return json.load(response)


def task(platform, chunk, key):
    return "index/v1/task/project.servo.%s_wpt_%s.%s" % (platform, chunk, key)


def failing_reftests(platform, key):
    chunk_1_task_id = fetch_json(task(platform, 1, key))["taskId"]
    name = fetch_json("queue/v1/task/" + chunk_1_task_id)["metadata"]["name"]
    match = re.search("WPT chunk (\d+) / (\d+)", name)
    assert match.group(1) == "1"
    total_chunks = int(match.group(2))

    for chunk in range(1, total_chunks + 1):
        with fetch(task(platform, chunk, key) + "/artifacts/public/test-wpt.log") as response:
            yield from parse(response)


def parse(file_like):
    seen = set()
    for line in file_like:
        message = json.loads(line)
        status = message.get("status")
        if status not in {None, "OK", "PASS"}:
            screenshots = message.get("extra", {}).get("reftest_screenshots")
            if screenshots:
                url = message["test"]
                assert url.startswith("/")
                yield url[1:], message.get("expected") == "PASS", screenshots


def main(source, commit_sha=None):
    failures = Directory()

    if commit_sha:
        title = "<h1>Layout 2020 regressions in commit <code>%s</code></h1>" % commit_sha
        failures_2013 = {url for url, _, _ in failing_reftests("linux_x64", source)}
        for url, _expected_pass, screenshots in failing_reftests("linux_x64_2020", source):
            if url not in failures_2013:
                failures.add(url, screenshots)
    else:
        title = "Unexpected failures"
        with open(source, "rb") as file_obj:
            for url, expected_pass, screenshots in parse(file_obj):
                if expected_pass:
                    failures.add(url, screenshots)

    here = os.path.dirname(__file__)
    with open(os.path.join(here, "prism.js")) as f:
        prism_js = f.read()
    with open(os.path.join(here, "prism.css")) as f:
        prism_css = f.read()
    with open(os.path.join(here, "report.html"), "w", encoding="utf-8") as html:
        os.chdir(os.path.join(here, ".."))
        html.write("""
            <!doctype html>
            <meta charset=utf-8>
            <title>WPT reftests failures report</title>
            <link rel=stylesheet href=prism.css>
            <style>
                ul { padding-left: 1em }
                li { list-style: "⯈ " }
                li.expanded { list-style: "⯆ " }
                li:not(.expanded) > ul, li:not(.expanded) > div { display: none }
                li > div { display: grid; grid-gap: 1em; grid-template-columns: 1fr 1fr }
                li > div > p { grid-column: span 2 }
                li > div > img { grid-row: 2; width: 300px; box-shadow: 0 0 10px }
                li > div > img:hover { transform: scale(3); transform-origin: 0 0 }
                li > div > pre { grid-row: 3; font-size: 12px !important }
                pre code { white-space: pre-wrap !important }
                <h1>%s</h1>
            </style>
            %s
        """ % (prism_css, title))
        failures.write(html)
        html.write("""
            <script>
                for (let li of document.getElementsByTagName("li")) {
                    li.addEventListener('click', event => {
                        li.classList.toggle("expanded")
                        event.stopPropagation()
                    })
                }
                %s
            </script>
        """ % prism_js)


class Directory:
    def __init__(self):
        self.count = 0
        self.contents = {}

    def add(self, path, screenshots):
        self.count += 1
        first, _, rest = path.partition("/")
        if rest:
            self.contents.setdefault(first, Directory()).add(rest, screenshots)
        else:
            assert path not in self.contents
            self.contents[path] = screenshots

    def write(self, html):
        html.write("<ul>\n")
        for k, v in self.contents.items():
            html.write("<li><code>%s</code>\n" % k)
            if isinstance(v, Directory):
                html.write("<strong>%s</strong>\n" % v.count)
                v.write(html)
            else:
                a, rel, b = v
                html.write("<div>\n<p><code>%s</code> %s <code>%s</code></p>\n"
                           % (a["url"], rel, b["url"]))
                for side in [a, b]:
                    html.write("<img src='data:image/png;base64,%s'>\n" % side["screenshot"])
                    url = side["url"]
                    prefix = "/_mozilla/"
                    if url.startswith(prefix):
                        filename = "mozilla/tests/" + url[len(prefix):]
                    elif url == "about:blank":
                        src = ""
                        filename = None
                    else:
                        filename = "web-platform-tests" + url
                    if filename:
                        with open(filename, encoding="utf-8") as f:
                            src = html_escape(f.read())
                    html.write("<pre><code class=language-html>%s</code></pre>\n" % src)
            html.write("</li>\n")
        html.write("</ul>\n")


if __name__ == "__main__":
    sys.exit(main(*sys.argv[1:]))
