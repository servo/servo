import requests
from mozlog.structured.formatters.base import BaseFormatter

DEFAULT_API = "https://wpt.fyi/api/screenshots/hashes"


class WptscreenshotFormatter(BaseFormatter):
    """Formatter that outputs screenshots in the format expected by wpt.fyi."""

    def __init__(self, api=None):
        self.api = api or DEFAULT_API
        self.cache = set()

    def suite_start(self, data):
        # TODO(Hexcles): We might want to move the request into a different
        # place, make it non-blocking, and handle errors better.
        params = {}
        run_info = data.get("run_info", {})
        if "product" in run_info:
            params["browser"] = run_info["product"]
        if "browser_version" in run_info:
            params["browser_version"] = run_info["browser_version"]
        if "os" in run_info:
            params["os"] = run_info["os"]
        if "os_version" in run_info:
            params["os_version"] = run_info["os_version"]
        try:
            r = requests.get(self.api, params=params)
            r.raise_for_status()
            self.cache = set(r.json())
        except (requests.exceptions.RequestException, ValueError):
            pass

    def test_end(self, data):
        if "reftest_screenshots" not in data.get("extra", {}):
            return
        output = ""
        for item in data["extra"]["reftest_screenshots"]:
            if type(item) != dict:
                # Skip the relation string.
                continue
            checksum = "sha1:" + item["hash"]
            if checksum in self.cache:
                continue
            self.cache.add(checksum)
            output += "data:image/png;base64,{}\n".format(item["screenshot"])
        return output if output else None
