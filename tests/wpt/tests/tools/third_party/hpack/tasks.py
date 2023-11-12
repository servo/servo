# -*- coding: utf-8 -*-
import json
import os

from binascii import hexlify
from invoke import task
from hyper.http20.hpack import Encoder


@task
def hpack():
    """
    This task generates HPACK test data suitable for use with
    https://github.com/http2jp/hpack-test-case

    The current format defines a JSON object with three keys: 'draft',
    'description' and 'cases'.

    The cases key has as its value a list of objects, with each object
    representing a set of headers and the output from the encoder. The object
    has the following keys:

    - 'header_table_size': the size of the header table used.
    - 'headers': A list of the headers as JSON objects.
    - 'wire': The output from the encoder in hexadecimal.
    """
    # A generator that contains the paths to all the raw data files and their
    # names.
    raw_story_files = (
        (os.path.join('test/test_fixtures/raw-data', name), name)
        for name in os.listdir('test/test_fixtures/raw-data')
    )

    # For each file, build our output.
    for source, outname in raw_story_files:
        with open(source, 'rb') as f:
            indata = json.load(f)

        # Prepare the output and the encoder.
        output = {
            'description': 'Encoded by hyper. See github.com/Lukasa/hyper for more information.',
            'cases': []
        }
        e = Encoder()

        for case in indata['cases']:
            outcase = {
                'header_table_size': e.header_table_size,
                'headers': case['headers'],
            }
            headers = []

            for header in case['headers']:
                key = header.keys()[0]
                header = (key, header[key])
                headers.append(header)

            outcase['wire'] = hexlify(e.encode(headers))
            output['cases'].append(outcase)

        with open(outname, 'wb') as f:
            f.write(json.dumps(output, sort_keys=True,
                    indent=2, separators=(',', ': ')))
