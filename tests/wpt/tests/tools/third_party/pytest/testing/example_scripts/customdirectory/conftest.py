# mypy: allow-untyped-defs
# content of conftest.py
import json

import pytest


class ManifestDirectory(pytest.Directory):
    def collect(self):
        manifest_path = self.path / "manifest.json"
        manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
        ihook = self.ihook
        for file in manifest["files"]:
            yield from ihook.pytest_collect_file(
                file_path=self.path / file, parent=self
            )


@pytest.hookimpl
def pytest_collect_directory(path, parent):
    if path.joinpath("manifest.json").is_file():
        return ManifestDirectory.from_parent(parent=parent, path=path)
    return None
