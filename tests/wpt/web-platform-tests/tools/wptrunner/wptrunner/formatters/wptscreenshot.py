from mozlog.structured.formatters.base import BaseFormatter


class WptscreenshotFormatter(BaseFormatter):
    """Formatter that outputs screenshots in the format expected by wpt.fyi."""

    def __init__(self):
        self.cache = set()

    def suite_start(self, data):
        # TODO: ask wpt.fyi for known hashes.
        pass

    def test_end(self, data):
        if "reftest_screenshots" not in data.get("extra", {}):
            return
        output = ""
        for item in data["extra"]["reftest_screenshots"]:
            if type(item) != dict or item["hash"] in self.cache:
                continue
            self.cache.add(item["hash"])
            output += "data:image/png;base64,{}\n".format(item["screenshot"])
        return output if output else None
