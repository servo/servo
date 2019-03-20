import json
import os
import platform
import stat
import subprocess
from collections import deque

from .sourcefile import SourceFile


def get_tree(tests_root, manifest, manifest_path, cache_root,
             working_copy=False, rebuild=False):
    tree = None
    if cache_root is None:
        cache_root = os.path.join(tests_root, ".wptcache")
    if not os.path.exists(cache_root):
        try:
            os.makedirs(cache_root)
        except IOError:
            cache_root = None

    if not working_copy:
        tree = Git.for_path(tests_root,
                            manifest.url_base,
                            manifest_path=manifest_path,
                            cache_path=cache_root,
                            rebuild=rebuild)
    if tree is None:
        tree = FileSystem(tests_root,
                          manifest.url_base,
                          manifest_path=manifest_path,
                          cache_path=cache_root,
                          rebuild=rebuild)
    return tree


class Git(object):
    def __init__(self, repo_root, url_base, cache_path, manifest_path=None,
                 rebuild=False):
        self.root = repo_root
        self.git = Git.get_func(repo_root)
        self.url_base = url_base
        # rebuild is a noop for now since we don't cache anything

    @staticmethod
    def get_func(repo_path):
        def git(cmd, *args):
            full_cmd = ["git", cmd] + list(args)
            try:
                return subprocess.check_output(full_cmd, cwd=repo_path, stderr=subprocess.STDOUT)
            except Exception as e:
                if platform.uname()[0] == "Windows" and isinstance(e, WindowsError):
                    full_cmd[0] = "git.bat"
                    return subprocess.check_output(full_cmd, cwd=repo_path, stderr=subprocess.STDOUT)
                else:
                    raise
        return git

    @classmethod
    def for_path(cls, path, url_base, cache_path, manifest_path=None, rebuild=False):
        git = Git.get_func(path)
        try:
            # this needs to be a command that fails if we aren't in a git repo
            git("rev-parse", "--show-toplevel")
        except (subprocess.CalledProcessError, OSError):
            return None
        else:
            return cls(path, url_base, cache_path,
                       manifest_path=manifest_path, rebuild=rebuild)

    def _local_changes(self):
        """get a set of files which have changed between HEAD and working copy"""
        changes = set()

        cmd = ["status", "-z", "--ignore-submodules=all"]
        data = self.git(*cmd)

        in_rename = False
        for line in data.split(b"\0")[:-1]:
            if in_rename:
                changes.add(line)
                in_rename = False
            else:
                status = line[:2]
                if b"R" in status or b"C" in status:
                    in_rename = True
                changes.add(line[3:])

        return changes

    def _show_file(self, path):
        path = os.path.relpath(os.path.abspath(path), self.root)
        return self.git("show", "HEAD:%s" % path)

    def __iter__(self):
        cmd = ["ls-tree", "-r", "-z", "HEAD"]
        local_changes = self._local_changes()
        for result in self.git(*cmd).split("\0")[:-1]:
            data, rel_path = result.rsplit("\t", 1)
            hash = data.split(" ", 3)[2]
            if rel_path in local_changes:
                contents = self._show_file(rel_path)
            else:
                contents = None
            yield SourceFile(self.root,
                             rel_path,
                             self.url_base,
                             hash,
                             contents=contents), True

    def dump_caches(self):
        pass


class FileSystem(object):
    def __init__(self, root, url_base, cache_path, manifest_path=None, rebuild=False):
        from gitignore import gitignore
        self.root = os.path.abspath(root)
        self.url_base = url_base
        self.ignore_cache = None
        self.mtime_cache = None
        if cache_path is not None:
            if manifest_path is not None:
                self.mtime_cache = MtimeCache(cache_path, root, manifest_path, rebuild)
            if gitignore.has_ignore(root):
                self.ignore_cache = GitIgnoreCache(cache_path, root, rebuild)
        self.path_filter = gitignore.PathFilter(self.root,
                                                extras=[".git/"],
                                                cache=self.ignore_cache)

    def __iter__(self):
        mtime_cache = self.mtime_cache
        for dirpath, dirnames, filenames in self.path_filter(walk(self.root)):
            for filename, path_stat in filenames:
                path = os.path.join(dirpath, filename)
                if mtime_cache is None or mtime_cache.updated(path, path_stat):
                    yield SourceFile(self.root, path, self.url_base), True
                else:
                    yield path, False

    def dump_caches(self):
        for cache in [self.mtime_cache, self.ignore_cache]:
            if cache is not None:
                cache.dump()


class CacheFile(object):
    file_name = None

    def __init__(self, cache_root, tests_root, rebuild=False):
        self.tests_root = tests_root
        if not os.path.exists(cache_root):
            os.makedirs(cache_root)
        self.path = os.path.join(cache_root, self.file_name)
        self.modified = False
        self.data = self.load(rebuild)

    def dump(self):
        if not self.modified:
            return
        with open(self.path, 'w') as f:
            json.dump(self.data, f, indent=1)

    def load(self, rebuild=False):
        data = {}
        try:
            if not rebuild:
                with open(self.path, 'r') as f:
                    data = json.load(f)
                data = self.check_valid(data)
        except IOError:
            pass
        return data

    def check_valid(self, data):
        """Check if the cached data is valid and return an updated copy of the
        cache containing only data that can be used."""
        return data


class MtimeCache(CacheFile):
    file_name = "mtime.json"

    def __init__(self, cache_root, tests_root, manifest_path, rebuild=False):
        self.manifest_path = manifest_path
        super(MtimeCache, self).__init__(cache_root, tests_root, rebuild=False)

    def updated(self, rel_path, stat):
        """Return a boolean indicating whether the file changed since the cache was last updated.

        This implicitly updates the cache with the new mtime data."""
        mtime = stat.st_mtime
        if mtime != self.data.get(rel_path):
            self.modified = True
            self.data[rel_path] = mtime
            return True
        return False

    def check_valid(self, data):
        if data.get("/tests_root") != self.tests_root:
            self.modified = True
        else:
            if self.manifest_path is not None and os.path.exists(self.manifest_path):
                mtime = os.path.getmtime(self.manifest_path)
                if data.get("/manifest_path") != [self.manifest_path, mtime]:
                    self.modified = True
            else:
                self.modified = True
        if self.modified:
            data = {}
            data["/tests_root"] = self.tests_root
        return data

    def dump(self):
        if self.manifest_path is None:
            raise ValueError
        if not os.path.exists(self.manifest_path):
            return
        mtime = os.path.getmtime(self.manifest_path)
        self.data["/manifest_path"] = [self.manifest_path, mtime]
        self.data["/tests_root"] = self.tests_root
        super(MtimeCache, self).dump()


class GitIgnoreCache(CacheFile):
    file_name = "gitignore.json"

    def check_valid(self, data):
        ignore_path = os.path.join(self.tests_root, ".gitignore")
        mtime = os.path.getmtime(ignore_path)
        if data.get("/gitignore_file") != [ignore_path, mtime]:
            self.modified = True
            data = {}
            data["/gitignore_file"] = [ignore_path, mtime]
        return data

    def __contains__(self, key):
        return key in self.data

    def __getitem__(self, key):
        return self.data[key]

    def __setitem__(self, key, value):
        if self.data.get(key) != value:
            self.modified = True
            self.data[key] = value


def walk(root):
    """Re-implementation of os.walk. Returns an iterator over
    (dirpath, dirnames, filenames), with some semantic differences
    to os.walk.

    This has a similar interface to os.walk, with the important difference
    that instead of lists of filenames and directory names, it yields
    lists of tuples of the form [(name, stat)] where stat is the result of
    os.stat for the file. That allows reusing the same stat data in the
    caller. It also always returns the dirpath relative to the root, with
    the root iself being returned as the empty string.

    Unlike os.walk the implementation is not recursive."""

    listdir = os.listdir
    get_stat = os.stat
    listdir = os.listdir
    join = os.path.join
    is_dir = stat.S_ISDIR
    is_link = stat.S_ISLNK
    relpath = os.path.relpath

    root = os.path.abspath(root)
    stack = deque([(root, "")])

    while stack:
        dir_path, rel_path = stack.popleft()
        try:
            # Note that listdir and error are globals in this module due
            # to earlier import-*.
            names = listdir(dir_path)
        except OSError:
            continue

        dirs, non_dirs = [], []
        for name in names:
            path = join(dir_path, name)
            try:
                path_stat = get_stat(path)
            except OSError:
                continue
            if is_dir(path_stat.st_mode):
                dirs.append((name, path_stat))
            else:
                non_dirs.append((name, path_stat))

        yield rel_path, dirs, non_dirs
        for name, path_stat in dirs:
            new_path = join(dir_path, name)
            if not is_link(path_stat.st_mode):
                stack.append((new_path, relpath(new_path, root)))
