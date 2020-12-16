"""Coverage plugin to add exclude lines based on the Python version."""

import sys

from coverage import CoveragePlugin


class MyConfigPlugin(CoveragePlugin):
    def configure(self, config):
        opt_name = 'report:exclude_lines'
        exclude_lines = config.get_option(opt_name)
        # Python >= 3.6 has os.PathLike.
        if sys.version_info >= (3, 6):
            exclude_lines.append('pragma: >=36')
        else:
            exclude_lines.append('pragma: <=35')
        config.set_option(opt_name, exclude_lines)


def coverage_init(reg, options):
    reg.add_configurer(MyConfigPlugin())
