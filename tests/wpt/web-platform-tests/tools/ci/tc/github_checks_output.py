import io
from six import ensure_text

MYPY = False
if MYPY:
    # MYPY is set to True when run under Mypy.
    from typing import AnyStr, Optional, Text


class GitHubChecksOutputter(object):
    """Provides a method to output data to be shown in the GitHub Checks UI.

    This can be useful to provide a summary of a given check (e.g. the lint)
    to enable developers to quickly understand what has gone wrong. The output
    supports markdown format.

    https://docs.taskcluster.net/docs/reference/integrations/github/checks#custom-text-output-in-checks
    """
    def __init__(self, path):
        # type: (Text) -> None
        self.path = path

    def output(self, line):
        # type: (AnyStr) -> None
        text = ensure_text(line)
        # NOTE: mypy types the "text mode" of open() in Python 2 as BinaryIO,
        # which makes sense as we cannot specify its encoding (it's
        # platform-dependent), while io.open() is closer to open() in Python 3.
        # TODO: use the built-in open() when we are Py3-only.
        with io.open(self.path, mode="a") as f:
            f.write(text)
            f.write(u"\n")


__outputter = None


def get_gh_checks_outputter(filepath):
    # type: (Optional[Text]) -> Optional[GitHubChecksOutputter]
    """Return the outputter for GitHub Checks output, if enabled.

    :param filepath: The filepath to write GitHub Check output information to,
                     or None if not enabled.
    """
    global __outputter
    if filepath and __outputter is None:
        __outputter = GitHubChecksOutputter(filepath)
    return __outputter
