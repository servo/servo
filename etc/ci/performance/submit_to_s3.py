#!/usr/bin/env python3

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.

import argparse
import boto3


def main():
    parser = argparse.ArgumentParser(
        description=("Submit Servo performance data to S3. "
                     "Remember to set your S3 credentials "
                     "https://github.com/boto/boto3"))
    parser.add_argument("perf_file",
                        help="the output CSV file from runner")
    parser.add_argument("perf_key",
                        help="the S3 key to upload to")
    args = parser.parse_args()

    s3 = boto3.client('s3')
    BUCKET = 'servo-perf'
    s3.upload_file(args.perf_file, BUCKET, args.perf_key)

    print("Done!")


if __name__ == "__main__":
    main()
