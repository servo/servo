import ConfigParser
import os
import sys
from collections import OrderedDict

here = os.path.split(__file__)[0]

class ConfigDict(dict):
    def __init__(self, base_path, *args, **kwargs):
        self.base_path = base_path
        dict.__init__(self, *args, **kwargs)

    def get_path(self, key, default=None):
        if key not in self:
            return default
        path = self[key]
        os.path.expanduser(path)
        return os.path.abspath(os.path.join(self.base_path, path))

def read(config_path):
    config_path = os.path.abspath(config_path)
    config_root = os.path.split(config_path)[0]
    parser = ConfigParser.SafeConfigParser()
    success = parser.read(config_path)
    assert config_path in success, success

    subns = {"pwd": os.path.abspath(os.path.curdir)}

    rv = OrderedDict()
    for section in parser.sections():
        rv[section] = ConfigDict(config_root)
        for key in parser.options(section):
            rv[section][key] = parser.get(section, key, False, subns)

    return rv

def path(argv=None):
    if argv is None:
        argv = []
    path = None

    for i, arg in enumerate(argv):
        if arg == "--config":
            if i + 1 < len(argv):
                path = argv[i + 1]
        elif arg.startswith("--config="):
            path = arg.split("=", 1)[1]
        if path is not None:
            break

    if path is None:
        if os.path.exists("wptrunner.ini"):
            path = os.path.abspath("wptrunner.ini")
        else:
            path = os.path.join(here, "..", "wptrunner.default.ini")

    return os.path.abspath(path)

def load():
    return read(path(sys.argv))
