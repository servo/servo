#!/usr/bin/env python3

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

import argparse
import csv
from datetime import datetime, date
import httplib2
import json
from math import floor
import os

SCRIPT_PATH = os.path.split(__file__)[0]


def main():
    default_output_dir = os.path.join(SCRIPT_PATH, 'output')
    default_cache_dir = os.path.join(SCRIPT_PATH, '.cache')

    parser = argparse.ArgumentParser(
        description="Download buildbot metadata"
    )
    parser.add_argument("--index-url",
                        type=str,
                        default='http://build.servo.org/json',
                        help="the URL to get the JSON index data index from. "
                        "Default: http://build.servo.org/json")
    parser.add_argument("--build-url",
                        type=str,
                        default='http://build.servo.org/json/builders/{}/builds/{}',
                        help="the URL to get the JSON build data from. "
                        "Default: http://build.servo.org/json/builders/{}/builds/{}")
    parser.add_argument("--cache-dir",
                        type=str,
                        default=default_cache_dir,
                        help="the directory to cache JSON files in. Default: " + default_cache_dir)
    parser.add_argument("--cache-name",
                        type=str,
                        default='build-{}-{}.json',
                        help="the filename to cache JSON data in. "
                        "Default: build-{}-{}.json")
    parser.add_argument("--output-dir",
                        type=str,
                        default=default_output_dir,
                        help="the directory to save the CSV data to. Default: " + default_output_dir)
    parser.add_argument("--output-name",
                        type=str,
                        default='builds-{}-{}.csv',
                        help="the filename to save the CSV data to. "
                        "Default: builds-{}-{}.csv")
    parser.add_argument("--verbose", "-v",
                        action='store_true',
                        help="print every HTTP request")
    args = parser.parse_args()

    http = httplib2.Http()

    os.makedirs(args.cache_dir, exist_ok=True)
    os.makedirs(args.output_dir, exist_ok=True)

    # Get the index to find out the list of builder names
    # Note: this isn't cached
    if args.verbose:
        print("Downloading index {}.".format(args.index_url))
    (index_headers, index_data) = http.request(args.index_url, "GET", headers={'cache-control': 'no-cache'})
    if args.verbose:
        print("Response {}.".format(index_headers))
    index = json.loads(index_data.decode('utf-8'))

    builds = []

    for builder in index["builders"]:
        # The most recent build is at offset -1
        # Fetch it to find out the build number
        # Note: this isn't cached
        recent_build_url = args.build_url.format(builder, -1)
        if args.verbose:
            print("Downloading recent build {}.".format(recent_build_url))
        (recent_build_headers, recent_build_data) = http.request(
            recent_build_url,
            "GET",
            headers={'cache-control': 'no-cache'}
        )
        if args.verbose:
            print("Respose {}.".format(recent_build_headers))
        recent_build = json.loads(recent_build_data.decode('utf-8'))
        recent_build_number = recent_build["number"]

        # Download each build, and convert to CSV
        for build_number in range(0, recent_build_number):

            # Rather annoyingly, we can't just use the Python http cache,
            # because it doesn't cache 404 responses. So we roll our own.
            cache_json_name = args.cache_name.format(builder, build_number)
            cache_json = os.path.join(args.cache_dir, cache_json_name)
            if os.path.isfile(cache_json):
                with open(cache_json) as f:
                    build = json.load(f)

            else:
                # Get the build data
                build_url = args.build_url.format(builder, build_number)
                if args.verbose:
                    print("Downloading build {}.".format(build_url))
                (build_headers, build_data) = http.request(
                    build_url,
                    "GET",
                    headers={'cache-control': 'no=cache'}
                )
                if args.verbose:
                    print("Response {}.".format(build_headers))

                # Only parse the JSON if we got back a 200 response.
                if build_headers.status == 200:
                    build = json.loads(build_data.decode('utf-8'))
                    # Don't cache current builds.
                    if build.get('currentStep'):
                        continue

                elif build_headers.status == 404:
                    build = {}

                else:
                    continue

                with open(cache_json, 'w+') as f:
                    json.dump(build, f)

            if 'times' in build:
                builds.append(build)

    years = {}
    for build in builds:
        build_date = date.fromtimestamp(build['times'][0])
        years.setdefault(build_date.year, {}).setdefault(build_date.month, []).append(build)

    for year, months in years.items():
        for month, builds in months.items():

            output_name = args.output_name.format(year, month)
            output = os.path.join(args.output_dir, output_name)

            # Create the CSV file.
            if args.verbose:
                print('Creating file {}.'.format(output))
            with open(output, 'w+') as output_file:
                output_csv = csv.writer(output_file)

                # The CSV column names
                output_csv.writerow([
                    'builder',
                    'buildNumber',
                    'buildTimestamp',
                    'stepName',
                    'stepText',
                    'stepNumber',
                    'stepStart',
                    'stepFinish'
                ])

                for build in builds:

                    builder = build["builderName"]
                    build_number = build["number"]
                    build_timestamp = datetime.fromtimestamp(build["times"][0]).replace(microsecond=0)

                    # Write out the timing data for each step
                    for step in build["steps"]:
                        if step["isFinished"]:
                            step_name = step["name"]
                            step_text = ' '.join(step["text"])
                            step_number = step["step_number"]
                            step_start = floor(step["times"][0])
                            step_finish = floor(step["times"][1])
                            output_csv.writerow([
                                builder,
                                build_number,
                                build_timestamp,
                                step_name,
                                step_text,
                                step_number,
                                step_start,
                                step_finish
                            ])


if __name__ == "__main__":
    main()
