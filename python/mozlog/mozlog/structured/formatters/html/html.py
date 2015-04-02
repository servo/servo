#!/usr/bin/env python
# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

import base64
import cgi
from datetime import datetime
import os

from .. import base

from collections import defaultdict

html = None
raw = None

base_path = os.path.split(__file__)[0]

def do_defered_imports():
    global html
    global raw

    from .xmlgen import html, raw


class HTMLFormatter(base.BaseFormatter):
    """Formatter that produces a simple HTML-formatted report."""
    def __init__(self):
        do_defered_imports()
        self.suite_name = None
        self.result_rows = []
        self.test_count = defaultdict(int)
        self.start_times = {}
        self.suite_times = {"start": None,
                            "end": None}
        self.head = None
        self.env = {}

    def suite_start(self, data):
        self.suite_times["start"] = data["time"]
        self.suite_name = data["source"]
        with open(os.path.join(base_path, "style.css")) as f:
            self.head = html.head(
                html.meta(charset="utf-8"),
                html.title(data["source"]),
                html.style(raw(f.read())))

        date_format = "%d %b %Y %H:%M:%S"
        version_info = data.get("version_info")
        if version_info:
            self.env["Device identifier"] = version_info.get("device_id")
            self.env["Device firmware (base)"] = version_info.get("device_firmware_version_base")
            self.env["Device firmware (date)"] = (
                datetime.utcfromtimestamp(int(version_info.get("device_firmware_date"))).strftime(date_format) if
                "device_firmware_date" in version_info else None)
            self.env["Device firmware (incremental)"] = version_info.get("device_firmware_version_incremental")
            self.env["Device firmware (release)"] = version_info.get("device_firmware_version_release")
            self.env["Gaia date"] = (
                datetime.utcfromtimestamp(int(version_info.get("gaia_date"))).strftime(date_format) if
                "gaia_date" in version_info else None)
            self.env["Gecko version"] = version_info.get("application_version")
            self.env["Gecko build"] = version_info.get("application_buildid")

            if version_info.get("application_changeset"):
                self.env["Gecko revision"] = version_info.get("application_changeset")
                if version_info.get("application_repository"):
                    self.env["Gecko revision"] = html.a(
                        version_info.get("application_changeset"),
                        href="/".join([version_info.get("application_repository"),
                                       version_info.get("application_changeset")]),
                        target="_blank")

            if version_info.get("gaia_changeset"):
                self.env["Gaia revision"] = html.a(
                    version_info.get("gaia_changeset")[:12],
                    href="https://github.com/mozilla-b2g/gaia/commit/%s" % version_info.get("gaia_changeset"),
                    target="_blank")

        device_info = data.get("device_info")
        if device_info:
            self.env["Device uptime"] = device_info.get("uptime")
            self.env["Device memory"] = device_info.get("memtotal")
            self.env["Device serial"] = device_info.get("id")

    def suite_end(self, data):
        self.suite_times["end"] = data["time"]
        return self.generate_html()

    def test_start(self, data):
        self.start_times[data["test"]] = data["time"]

    def test_end(self, data):
        self.make_result_html(data)

    def make_result_html(self, data):
        tc_time = (data["time"] - self.start_times.pop(data["test"])) / 1000.
        additional_html = []
        debug = data.get("extra", {})
        links_html = []

        status = status_name = data["status"]
        expected = data.get("expected", status)

        if status != expected:
            status_name = "UNEXPECTED_" + status
        elif status not in ("PASS", "SKIP"):
            status_name = "EXPECTED_" + status

        self.test_count[status_name] += 1

        if status in ['SKIP', 'FAIL', 'ERROR']:
            if debug.get('screenshot'):
                screenshot = 'data:image/png;base64,%s' % debug['screenshot']
                additional_html.append(html.div(
                    html.a(html.img(src=screenshot), href="#"),
                    class_='screenshot'))
            for name, content in debug.items():
                if 'screenshot' in name:
                    href = '#'
                else:
                    # use base64 to avoid that some browser (such as Firefox, Opera)
                    # treats '#' as the start of another link if the data URL contains.
                    # use 'charset=utf-8' to show special characters like Chinese.
                    href = 'data:text/plain;charset=utf-8;base64,%s' % base64.b64encode(content.encode('utf-8'))
                links_html.append(html.a(
                    name.title(),
                    class_=name,
                    href=href,
                    target='_blank'))
                links_html.append(' ')

            log = html.div(class_='log')
            output = data.get('stack', '').splitlines()
            output.extend(data.get('message', '').splitlines())
            for line in output:
                separator = line.startswith(' ' * 10)
                if separator:
                    log.append(line[:80])
                else:
                    if line.lower().find("error") != -1 or line.lower().find("exception") != -1:
                        log.append(html.span(raw(cgi.escape(line)), class_='error'))
                    else:
                        log.append(raw(cgi.escape(line)))
                log.append(html.br())
            additional_html.append(log)

        self.result_rows.append(
            html.tr([html.td(status_name, class_='col-result'),
                     html.td(data['test'], class_='col-name'),
                     html.td('%.2f' % tc_time, class_='col-duration'),
                     html.td(links_html, class_='col-links'),
                     html.td(additional_html, class_='debug')],
                    class_=status_name.lower() + ' results-table-row'))

    def generate_html(self):
        generated = datetime.utcnow()
        with open(os.path.join(base_path, "main.js")) as main_f:
            doc = html.html(
                self.head,
                html.body(
                    html.script(raw(main_f.read())),
                    html.p('Report generated on %s at %s' % (
                        generated.strftime('%d-%b-%Y'),
                        generated.strftime('%H:%M:%S'))),
                    html.h2('Environment'),
                    html.table(
                        [html.tr(html.td(k), html.td(v)) for k, v in sorted(self.env.items()) if v],
                        id='environment'),

                    html.h2('Summary'),
                    html.p('%i tests ran in %.1f seconds.' % (sum(self.test_count.itervalues()),
                                                              (self.suite_times["end"] -
                                                               self.suite_times["start"]) / 1000.),
                           html.br(),
                           html.span('%i passed' % self.test_count["PASS"], class_='pass'), ', ',
                           html.span('%i skipped' % self.test_count["SKIP"], class_='skip'), ', ',
                           html.span('%i failed' % self.test_count["UNEXPECTED_FAIL"], class_='fail'), ', ',
                           html.span('%i errors' % self.test_count["UNEXPECTED_ERROR"], class_='error'), '.',
                           html.br(),
                           html.span('%i expected failures' % self.test_count["EXPECTED_FAIL"],
                                     class_='expected_fail'), ', ',
                           html.span('%i unexpected passes' % self.test_count["UNEXPECTED_PASS"],
                                     class_='unexpected_pass'), '.'),
                    html.h2('Results'),
                    html.table([html.thead(
                        html.tr([
                            html.th('Result', class_='sortable', col='result'),
                            html.th('Test', class_='sortable', col='name'),
                            html.th('Duration', class_='sortable numeric', col='duration'),
                            html.th('Links')]), id='results-table-head'),
                        html.tbody(self.result_rows, id='results-table-body')], id='results-table')))

        return u"<!DOCTYPE html>\n" + doc.unicode(indent=2)
