#!/usr/bin/env python

import os, json
from common_paths import *
import spec_validator

def rmtree(top):
    top = os.path.abspath(top)
    assert top != os.path.expanduser("~")
    assert len(top) > len(os.path.expanduser("~"))
    assert "web-platform-tests" in top
    assert "mixed-content" in top

    for root, dirs, files in os.walk(top, topdown=False):
        for name in files:
            os.remove(os.path.join(root, name))
        for name in dirs:
            os.rmdir(os.path.join(root, name))

    os.rmdir(top)

def main():
    spec_json = load_spec_json();
    spec_validator.assert_valid_spec_json(spec_json)

    for spec in spec_json['specification']:
        generated_dir = os.path.join(spec_directory, spec["name"])
        if (os.path.isdir(generated_dir)):
            rmtree(generated_dir)

    if (os.path.isfile(generated_spec_json_filename)):
        os.remove(generated_spec_json_filename)

if __name__ == '__main__':
    main()
