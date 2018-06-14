import imp
import json
import os

here = os.path.dirname(__file__)
localpaths = imp.load_source("localpaths", os.path.abspath(os.path.join(here, os.pardir, "localpaths.py")))

root = localpaths.repo_root

from manifest import manifest, update

def main(request, response):
    path = os.path.join(root, "MANIFEST.json")

    manifest_file = None
    try:
        manifest_file = manifest.load(root, path)
    except manifest.ManifestVersionMismatch:
        pass
    if manifest_file is None:
        manifest_file = manifest.Manifest("/")

    update.update(root, manifest_file)

    manifest.write(manifest_file, path)

    return [("Content-Type", "application/json")], json.dumps({"url": "/MANIFEST.json"})
