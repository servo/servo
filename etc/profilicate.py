#!/usr/bin/env python3

# Copyright 2018 The Servo Project Developers. See the COPYRIGHT
# file at the top-level directory of this distribution.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

# Script to take raw sample output from Servo sampling profiler and
# output a [processed profile]. Based largely on [this script] and
# [this documentation].
#
# [processed profile]: https://github.com/firefox-devtools/profiler/blob/main/docs-developer/processed-profile-format.md
# [this script]: https://github.com/firefox-devtools/profiler/blob/main/src/profile-logic/import/linux-perf.js
# [this documentation]: https://github.com/firefox-devtools/profiler/blob/main/src/types/profile.js

from collections import defaultdict
import json
import sys


class StringTable:
    def __init__(self):
        self.table = {}
        self.idx = 0

    def get(self, s):
        assert s
        if s not in self.table:
            self.table[s] = self.idx
            self.idx += 1
        return self.table[s]

    def length(self):
        return len(list(self.table.keys()))

    def contents(self):
        return sorted(list(self.table.keys()), key=self.table.__getitem__)


with open(sys.argv[1]) as f:
    profile = json.load(f)
    rate = profile["rate"]
    samples = profile["data"]
    startTime = profile["start"]

frames = {}
stacks = {}

thread_data = defaultdict(list)
thread_order = {}
for sample in samples:
    if sample['name']:
        name = sample['name']
    else:
        name = "%s %d %d" % (sample['type'], sample['namespace'], sample['index'])
    thread_data[name].append((sample['time'], sample['frames']))
    if name not in thread_order:
        thread_order[name] = (sample['namespace'], sample['index'])

tid = 0
threads = []
for (name, raw_samples) in sorted(iter(thread_data.items()), key=lambda x: thread_order[x[0]]):
    string_table = StringTable()
    tid += 1

    stackMap = {}
    stacks = []
    frameMap = {}
    frames = []

    samples = []

    for sample in raw_samples:
        prefix = None
        for frame in sample[1]:
            if not frame['name']:
                continue
            if not frame['name'] in frameMap:
                frameMap[frame['name']] = len(frames)
                frame_index = string_table.get(frame['name'])
                frames.append([frame_index])
            frame = frameMap[frame['name']]

            stack_key = "%d,%d" % (frame, prefix) if prefix else str(frame)
            if stack_key not in stackMap:
                stackMap[stack_key] = len(stacks)
                stacks.append([frame, prefix])
            stack = stackMap[stack_key]
            prefix = stack
        samples.append([stack, sample[0]])

    threads.append({
        'tid': tid,
        'name': name,
        'markers': {
            'schema': {
                'name': 0,
                'time': 1,
                'data': 2,
            },
            'data': [],
        },
        'samples': {
            'schema': {
                'stack': 0,
                'time': 1,
                'responsiveness': 2,
                'rss': 2,
                'uss': 4,
                'frameNumber': 5,
            },
            'data': samples,
        },
        'frameTable': {
            'schema': {
                'location': 0,
                'implementation': 1,
                'optimizations': 2,
                'line': 3,
                'category': 4,
            },
            'data': frames,
        },
        'stackTable': {
            'schema': {
                'frame': 0,
                'prefix': 1,
            },
            'data': stacks,
        },
        'stringTable': string_table.contents(),
    })


output = {
    'meta': {
        'interval': rate,
        'processType': 0,
        'product': 'Servo',
        'stackwalk': 1,
        'startTime': startTime,
        'version': 4,
        'presymbolicated': True,
    },
    'libs': [],
    'threads': threads,
}

print(json.dumps(output))
