# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

from __future__ import print_function
import json
import os
import subprocess
import time
import urllib2

FILE = 'perf.json'
URL = 'https://wrperf.org/submit'

while True:
    try:
        # Remove any previous results
        try:
            os.remove(FILE)
        except:
            pass

        # Pull latest code
        subprocess.call(["git", "pull"])

        # Get the git revision of this build
        revision = subprocess.check_output(["git", "rev-parse", "HEAD"]).strip()

        # Build
        subprocess.call(["cargo", "build", "--release"])

        # Run benchmarks
        env = os.environ.copy()
        # Ensure that vsync is disabled, to get meaningful 'composite' times.
        env['vblank_mode'] = '0'
        subprocess.call(["cargo", "run", "--release", "--", "perf", FILE], env=env)

        # Read the results
        with open(FILE) as file:
            results = json.load(file)

        # Post the results to server
        payload = {
            'key': env['WEBRENDER_PERF_KEY'],
            'revision': revision,
            'timestamp': str(time.time()),
            'tests': results['tests'],
        }

        req = urllib2.Request(URL,
                              headers={"Content-Type": "application/json"},
                              data=json.dumps(payload))

        f = urllib2.urlopen(req)
    except Exception as e:
        print(e)

    # Delay a bit until next benchmark
    time.sleep(60 * 60)
