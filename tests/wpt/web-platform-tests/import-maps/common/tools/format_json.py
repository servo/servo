import collections
import json
import sys
import traceback
"""
Simple JSON formatter, to be used for JSON files under resources/.

Usage:
$ python tools/format_json.py resources/*.json
"""


def main():
    for filename in sys.argv[1:]:
        print filename
        try:
            spec = json.load(
                open(filename, 'r'), object_pairs_hook=collections.OrderedDict)
            with open(filename, 'w') as f:
                f.write(json.dumps(spec, indent=2, separators=(',', ': ')))
                f.write('\n')
        except:
            traceback.print_exc()


if __name__ == '__main__':
    main()
