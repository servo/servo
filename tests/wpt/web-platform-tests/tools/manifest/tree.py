import os
from cStringIO import StringIO
from fnmatch import fnmatch

import vcs
from log import get_logger
from utils import is_blacklisted, rel_path_to_url

def chunks(data, n):
    for i in range(0, len(data) - 1, n):
        yield data[i:i+n]

class TestTree(object):
    def __init__(self, tests_root, url_base):
        self.tests_root = tests_root
        self.url_base = url_base
        self.logger = get_logger()

    def current_rev(self):
        pass

    def local_changes(self):
        pass

    def committed_changes(self, base_rev=None):
        pass


class GitTree(TestTree):
    def __init__(self, tests_root, url_base):
        TestTree.__init__(self, tests_root, url_base)
        self.git = self.setup_git()

    def setup_git(self):
        assert vcs.is_git_repo(self.tests_root)
        return vcs.get_git_func(self.tests_root)

    def current_rev(self):
        return self.git("rev-parse", "HEAD").strip()

    def local_changes(self, path=None):
        # -z is stable like --porcelain; see the git status documentation for details
        cmd = ["status", "-z", "--ignore-submodules=all"]
        if path is not None:
            cmd.extend(["--", path])

        rv = {}

        data = self.git(*cmd)
        if data == "":
            return rv

        assert data[-1] == "\0"
        f = StringIO(data)

        while f.tell() < len(data):
            # First two bytes are the status in the stage (index) and working tree, respectively
            staged = f.read(1)
            worktree = f.read(1)
            assert f.read(1) == " "

            if staged == "R":
                # When a file is renamed, there are two files, the source and the destination
                files = 2
            else:
                files = 1

            filenames = []

            for i in range(files):
                filenames.append("")
                char = f.read(1)
                while char != "\0":
                    filenames[-1] += char
                    char = f.read(1)

            if not is_blacklisted(rel_path_to_url(filenames[0], self.url_base)):
                rv.update(self.local_status(staged, worktree, filenames))

        return rv

    def committed_changes(self, base_rev=None):
        if base_rev is None:
            self.logger.debug("Adding all changesets to the manifest")
            return [(item, "modified") for item in self.paths()]

        self.logger.debug("Updating the manifest from %s to %s" % (base_rev, self.current_rev()))
        rv = []
        data  = self.git("diff", "-z", "--name-status", base_rev + "..HEAD")
        items = data.split("\0")
        for status, filename in chunks(items, 2):
            if is_blacklisted(rel_path_to_url(filename, self.url_base)):
                continue
            if status == "D":
                rv.append((filename, "deleted"))
            else:
                rv.append((filename, "modified"))
        return rv

    def paths(self):
        data = self.git("ls-tree", "--name-only", "--full-tree", "-r", "HEAD")
        return [item for item in data.split("\n") if not item.endswith(os.path.sep)]

    def local_status(self, staged, worktree, filenames):
        # Convert the complex range of statuses that git can have to two values
        # we care about; "modified" and "deleted" and return a dictionary mapping
        # filenames to statuses

        rv = {}

        if (staged, worktree) in [("D", "D"), ("A", "U"), ("U", "D"), ("U", "A"),
                                  ("D", "U"), ("A", "A"), ("U", "U")]:
            raise Exception("Can't operate on tree containing unmerged paths")

        if staged == "R":
            assert len(filenames) == 2
            dest, src = filenames
            rv[dest] = "modified"
            rv[src] = "deleted"
        else:
            assert len(filenames) == 1

            filename = filenames[0]

            if staged == "D" or worktree == "D":
                # Actually if something is deleted in the index but present in the worktree
                # it will get included by having a status of both "D " and "??".
                # It isn't clear whether that's a bug
                rv[filename] = "deleted"
            elif staged == "?" and worktree == "?":
                # A new file. If it's a directory, recurse into it
                if os.path.isdir(os.path.join(self.tests_root, filename)):
                    rv.update(self.local_changes(filename))
                else:
                    rv[filename] = "modified"
            else:
                rv[filename] = "modified"

        return rv

class NoVCSTree(TestTree):
    """Subclass that doesn't depend on git"""

    ignore = ["*.py[c|0]", "*~", "#*"]

    def current_rev(self):
        return None

    def local_changes(self):
        # Put all files into local_changes and rely on Manifest.update to de-dupe
        # changes that in fact committed at the base rev.

        rv = []
        for dir_path, dir_names, filenames in os.walk(self.tests_root):
            for filename in filenames:
                if any(fnmatch(filename, pattern) for pattern in self.ignore):
                    continue
                rel_path = os.path.relpath(os.path.join(dir_path, filename),
                                           self.tests_root)
                if is_blacklisted(rel_path_to_url(rel_path, self.url_base)):
                    continue
                rv.append((rel_path, "modified"))
        return dict(rv)

    def committed_changes(self, base_rev=None):
        return None
