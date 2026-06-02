import collections
import json
import os


def main():
    '''Formats spec.src.json.'''
    script_directory = os.path.dirname(os.path.abspath(__file__))
    for dir in [
            'mixed-content', 'referrer-policy', 'referrer-policy/4K-1',
            'referrer-policy/4K', 'referrer-policy/4K+1',
            'upgrade-insecure-requests'
    ]:
        filename = os.path.join(script_directory, '..', '..', '..', dir,
                                'spec.src.json')
        spec = json.load(
            open(filename, 'r'), object_pairs_hook=collections.OrderedDict)
        with open(filename, 'w') as f:
            f.write(json.dumps(spec, indent=2, separators=(',', ': ')))
            f.write('\n')


if __name__ == '__main__':
    main()
