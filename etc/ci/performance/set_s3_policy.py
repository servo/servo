#!/usr/bin/env python3

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.

import argparse
import boto3


def main():
    parser = argparse.ArgumentParser(
        description=("Set the policy of the servo-perf bucket. "
                     "Remember to set your S3 credentials "
                     "https://github.com/boto/boto3"))
    parser.parse_args()

    s3 = boto3.resource('s3')
    BUCKET = 'servo-perf'
    POLICY = """{
  "Version":"2012-10-17",
  "Statement":[
    {
      "Effect":"Allow",
      "Principal":"*",
      "Action":[
        "s3:ListBucket",
        "s3:GetBucketLocation"
      ],
      "Resource":"arn:aws:s3:::servo-perf"
    },
    {
      "Effect":"Allow",
      "Principal":"*",
      "Action":[
        "s3:GetObject",
        "s3:GetObjectAcl"
      ],
      "Resource":"arn:aws:s3:::servo-perf/*"
    }
  ]
}"""

    s3.BucketPolicy(BUCKET).put(Policy=POLICY)

    print("Done!")


if __name__ == "__main__":
    main()
