# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

import argparse
import cStringIO
import gzip
import json
import os
import requests
import urlparse

treeherder_base = "https://treeherder.mozilla.org/"

"""Simple script for downloading structured logs from treeherder.

For the moment this is specialised to work with web-platform-tests
logs; in due course it should move somewhere generic and get hooked
up to mach or similar"""

# Interpretation of the "job" list from
# https://github.com/mozilla/treeherder-service/blob/master/treeherder/webapp/api/utils.py#L18

def create_parser():
    parser = argparse.ArgumentParser()
    parser.add_argument("branch", action="store",
                        help="Branch on which jobs ran")
    parser.add_argument("commit",
                        action="store",
                        help="Commit hash for push")

    return parser

def download(url, prefix, dest, force_suffix=True):
    if dest is None:
        dest = "."

    if prefix and not force_suffix:
        name = os.path.join(dest, prefix + ".log")
    else:
        name = None
    counter = 0

    while not name or os.path.exists(name):
        counter += 1
        sep = "" if not prefix else "-"
        name = os.path.join(dest, prefix + sep + str(counter) + ".log")

    with open(name, "wb") as f:
        resp = requests.get(url, stream=True)
        for chunk in resp.iter_content(1024):
            f.write(chunk)

def get_blobber_url(branch, job):
    job_id = job["id"]
    resp = requests.get(urlparse.urljoin(treeherder_base,
                                         "/api/project/%s/artifact/?job_id=%i&name=Job%%20Info" % (branch,
                                                                                                   job_id)))
    job_data = resp.json()

    if job_data:
        assert len(job_data) == 1
        job_data = job_data[0]
        try:
            details = job_data["blob"]["job_details"]
            for item in details:
                if item["value"] == "wpt_raw.log":
                    return item["url"]
        except:
            return None


def get_structured_logs(branch, commit, dest=None):
    resp = requests.get(urlparse.urljoin(treeherder_base, "/api/project/%s/resultset/?revision=%s" % (branch, commit)))

    revision_data = resp.json()

    result_set = revision_data["results"][0]["id"]

    resp = requests.get(urlparse.urljoin(treeherder_base, "/api/project/%s/jobs/?result_set_id=%s&count=2000&exclusion_profile=false" % (branch, result_set)))

    job_data = resp.json()

    for result in job_data["results"]:
        job_type_name = result["job_type_name"]
        if job_type_name.startswith("W3C Web Platform"):
            url = get_blobber_url(branch, result)
            if url:
                prefix = result["platform"] # platform
                download(url, prefix, None)

def main():
    parser = create_parser()
    args = parser.parse_args()

    get_structured_logs(args.branch, args.commit)

if __name__ == "__main__":
    main()
