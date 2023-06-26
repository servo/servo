from typing import Optional, Text


class GitHubChecksOutputter:
    """Provides a method to output data to be shown in the GitHub Checks UI.

    This can be useful to provide a summary of a given check (e.g. the lint)
    to enable developers to quickly understand what has gone wrong. The output
    supports markdown format.

    https://docs.taskcluster.net/docs/reference/integrations/github/checks#custom-text-output-in-checks
    """
    def __init__(self, path: Text) -> None:
        self.path = path

    def output(self, line: Text) -> None:
        with open(self.path, mode="a") as f:
            f.write(line)
            f.write("\n")


__outputter = None


def get_gh_checks_outputter(filepath: Optional[Text]) -> Optional[GitHubChecksOutputter]:
    """Return the outputter for GitHub Checks output, if enabled.

    :param filepath: The filepath to write GitHub Check output information to,
                     or None if not enabled.
    """
    global __outputter
    if filepath and __outputter is None:
        __outputter = GitHubChecksOutputter(filepath)
    return __outputter
