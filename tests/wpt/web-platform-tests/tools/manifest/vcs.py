import abc
import os
import stat
from collections import deque
from collections.abc import MutableMapping

from . import jsonlib
from .utils import git

# Cannot do `from ..gitignore import gitignore` because
# relative import beyond toplevel throws *ImportError*!
from gitignore import gitignore  # type: ignore


MYPY = False
if MYPY:
    # MYPY is set to True when run under Mypy.
    from typing import Dict, Optional, List, Set, Text, Iterable, Any, Tuple, Iterator
    from .manifest import Manifest  # cyclic import under MYPY guard
    stat_result = os.stat_result

    GitIgnoreCacheType = MutableMapping[bytes, bool]
else:
    GitIgnoreCacheType = MutableMapping


def get_tree(tests_root, manifest, manifest_path, cache_root,
             working_copy=True, rebuild=False):
    # type: (Text, Manifest, Optional[Text], Optional[Text], bool, bool) -> FileSystem
    tree = None
    if cache_root is None:
        cache_root = os.path.join(tests_root, ".wptcache")
    if not os.path.exists(cache_root):
        try:
            os.makedirs(cache_root)
        except OSError:
            cache_root = None

    if not working_copy:
        raise ValueError("working_copy=False unsupported")

    if tree is None:
        tree = FileSystem(tests_root,
                          manifest.url_base,
                          manifest_path=manifest_path,
                          cache_path=cache_root,
                          rebuild=rebuild)
    return tree


class GitHasher:
    def __init__(self, path):
        # type: (Text) -> None
        self.git = git(path)

    def _local_changes(self):
        # type: () -> Set[Text]
        """get a set of files which have changed between HEAD and working copy"""
        assert self.git is not None
        # note that git runs the command with tests_root as the cwd, which may
        # not be the root of the git repo (e.g., within a browser repo)
        cmd = ["diff-index", "--relative", "--no-renames", "--name-only", "-z", "HEAD"]
        data = self.git(*cmd)
        return set(data.split("\0"))

    def hash_cache(self):
        # type: () -> Dict[Text, Optional[Text]]
        """
        A dict of rel_path -> current git object id if the working tree matches HEAD else None
        """
        hash_cache = {}  # type: Dict[Text, Optional[Text]]

        if self.git is None:
            return hash_cache

        # note that git runs the command with tests_root as the cwd, which may
        # not be the root of the git repo (e.g., within a browser repo)
        cmd = ["ls-tree", "-r", "-z", "HEAD"]
        local_changes = self._local_changes()
        for result in self.git(*cmd).split("\0")[:-1]:  # type: Text
            data, rel_path = result.rsplit("\t", 1)
            hash_cache[rel_path] = None if rel_path in local_changes else data.split(" ", 3)[2]

        return hash_cache



class FileSystem:
    def __init__(self, tests_root, url_base, cache_path, manifest_path=None, rebuild=False):
        # type: (Text, Text, Optional[Text], Optional[Text], bool) -> None
        self.tests_root = tests_root
        self.url_base = url_base
        self.ignore_cache = None
        self.mtime_cache = None
        tests_root_bytes = tests_root.encode("utf8")
        if cache_path is not None:
            if manifest_path is not None:
                self.mtime_cache = MtimeCache(cache_path, tests_root, manifest_path, rebuild)
            if gitignore.has_ignore(tests_root_bytes):
                self.ignore_cache = GitIgnoreCache(cache_path, tests_root, rebuild)
        self.path_filter = gitignore.PathFilter(tests_root_bytes,
                                                extras=[b".git/"],
                                                cache=self.ignore_cache)
        git = GitHasher(tests_root)
        self.hash_cache = git.hash_cache()

    def __iter__(self):
        # type: () -> Iterator[Tuple[Text, Optional[Text], bool]]
        mtime_cache = self.mtime_cache
        for dirpath, dirnames, filenames in self.path_filter(
                walk(self.tests_root.encode("utf8"))):
            for filename, path_stat in filenames:
                path = os.path.join(dirpath, filename).decode("utf8")
                if mtime_cache is None or mtime_cache.updated(path, path_stat):
                    file_hash = self.hash_cache.get(path, None)
                    yield path, file_hash, True
                else:
                    yield path, None, False

    def dump_caches(self):
        # type: () -> None
        for cache in [self.mtime_cache, self.ignore_cache]:
            if cache is not None:
                cache.dump()


class CacheFile(metaclass=abc.ABCMeta):
    def __init__(self, cache_root, tests_root, rebuild=False):
        # type: (Text, Text, bool) -> None
        self.tests_root = tests_root
        if not os.path.exists(cache_root):
            os.makedirs(cache_root)
        self.path = os.path.join(cache_root, self.file_name)
        self.modified = False
        self.data = self.load(rebuild)

    @abc.abstractproperty
    def file_name(self):
        # type: () -> Text
        pass

    def dump(self):
        # type: () -> None
        if not self.modified:
            return
        with open(self.path, 'w') as f:
            jsonlib.dump_local(self.data, f)

    def load(self, rebuild=False):
        # type: (bool) -> Dict[Text, Any]
        data = {}  # type: Dict[Text, Any]
        try:
            if not rebuild:
                with open(self.path) as f:
                    try:
                        data = jsonlib.load(f)
                    except ValueError:
                        pass
                data = self.check_valid(data)
        except OSError:
            pass
        return data

    def check_valid(self, data):
        # type: (Dict[Text, Any]) -> Dict[Text, Any]
        """Check if the cached data is valid and return an updated copy of the
        cache containing only data that can be used."""
        return data


class MtimeCache(CacheFile):
    file_name = "mtime.json"

    def __init__(self, cache_root, tests_root, manifest_path, rebuild=False):
        # type: (Text, Text, Text, bool) -> None
        self.manifest_path = manifest_path
        super().__init__(cache_root, tests_root, rebuild)

    def updated(self, rel_path, stat):
        # type: (Text, stat_result) -> bool
        """Return a boolean indicating whether the file changed since the cache was last updated.

        This implicitly updates the cache with the new mtime data."""
        mtime = stat.st_mtime
        if mtime != self.data.get(rel_path):
            self.modified = True
            self.data[rel_path] = mtime
            return True
        return False

    def check_valid(self, data):
        # type: (Dict[Any, Any]) -> Dict[Any, Any]
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
        # type: () -> None
        if self.manifest_path is None:
            raise ValueError
        if not os.path.exists(self.manifest_path):
            return
        mtime = os.path.getmtime(self.manifest_path)
        self.data["/manifest_path"] = [self.manifest_path, mtime]
        self.data["/tests_root"] = self.tests_root
        super().dump()


class GitIgnoreCache(CacheFile, GitIgnoreCacheType):
    file_name = "gitignore2.json"

    def check_valid(self, data):
        # type: (Dict[Any, Any]) -> Dict[Any, Any]
        ignore_path = os.path.join(self.tests_root, ".gitignore")
        mtime = os.path.getmtime(ignore_path)
        if data.get("/gitignore_file") != [ignore_path, mtime]:
            self.modified = True
            data = {}
            data["/gitignore_file"] = [ignore_path, mtime]
        return data

    def __contains__(self, key):
        # type: (Any) -> bool
        try:
            key = key.decode("utf-8")
        except Exception:
            return False

        return key in self.data

    def __getitem__(self, key):
        # type: (bytes) -> bool
        real_key = key.decode("utf-8")
        v = self.data[real_key]
        assert isinstance(v, bool)
        return v

    def __setitem__(self, key, value):
        # type: (bytes, bool) -> None
        real_key = key.decode("utf-8")
        if self.data.get(real_key) != value:
            self.modified = True
            self.data[real_key] = value

    def __delitem__(self, key):
        # type: (bytes) -> None
        real_key = key.decode("utf-8")
        del self.data[real_key]

    def __iter__(self):
        # type: () -> Iterator[bytes]
        return (key.encode("utf-8") for key in self.data)

    def __len__(self):
        # type: () -> int
        return len(self.data)


def walk(root):
    # type: (bytes) -> Iterable[Tuple[bytes, List[Tuple[bytes, stat_result]], List[Tuple[bytes, stat_result]]]]
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

    get_stat = os.stat
    is_dir = stat.S_ISDIR
    is_link = stat.S_ISLNK
    join = os.path.join
    listdir = os.listdir
    relpath = os.path.relpath

    root = os.path.abspath(root)
    stack = deque([(root, b"")])

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
