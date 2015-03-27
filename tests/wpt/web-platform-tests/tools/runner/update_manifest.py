import imp
import json
import os
import sys

here = os.path.dirname(__file__)
localpaths = imp.load_source("localpaths", os.path.abspath(os.path.join(here, os.pardir, "localpaths.py")))

root = localpaths.repo_root

import manifest

def main(request, response):
    path = os.path.join(root, "MANIFEST.json")
    manifest_file = manifest.manifest.load(root, path)
    manifest.update.update(root, "/", manifest_file)
    manifest.manifest.write(manifest_file, path)

    return [("Content-Type", "application/json")], json.dumps({"url": "/MANIFEST.json"})
