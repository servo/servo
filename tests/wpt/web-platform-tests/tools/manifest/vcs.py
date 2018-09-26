import os
import subprocess
import platform

from .sourcefile import SourceFile


class Git(object):
    def __init__(self, repo_root, url_base):
        self.root = os.path.abspath(repo_root)
        self.git = Git.get_func(repo_root)
        self.url_base = url_base

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
    def for_path(cls, path, url_base):
        git = Git.get_func(path)
        try:
            return cls(git("rev-parse", "--show-toplevel").rstrip(), url_base)
        except subprocess.CalledProcessError:
            return None

    def _local_changes(self):
        changes = {}
        cmd = ["status", "-z", "--ignore-submodules=all"]
        data = self.git(*cmd)

        if data == "":
            return changes

        rename_data = None
        for entry in data.split("\0")[:-1]:
            if rename_data is not None:
                status, rel_path = entry.split(" ")
                if status[0] == "R":
                    rename_data = (rel_path, status)
                else:
                    changes[rel_path] = (status, None)
            else:
                rel_path = entry
                changes[rel_path] = rename_data
                rename_data = None
        return changes

    def _show_file(self, path):
        path = os.path.relpath(os.path.abspath(path), self.root)
        return self.git("show", "HEAD:%s" % path)

    def __iter__(self):
        cmd = ["ls-tree", "-r", "-z", "HEAD"]
        local_changes = self._local_changes()
        for result in self.git(*cmd).split("\0")[:-1]:
            rel_path = result.split("\t")[-1]
            hash = result.split()[2]
            if not os.path.isdir(os.path.join(self.root, rel_path)):
                if rel_path in local_changes:
                    contents = self._show_file(rel_path)
                else:
                    contents = None
                yield SourceFile(self.root,
                                 rel_path,
                                 self.url_base,
                                 hash,
                                 contents=contents)


class FileSystem(object):
    def __init__(self, root, url_base):
        self.root = root
        self.url_base = url_base
        from gitignore import gitignore
        self.path_filter = gitignore.PathFilter(self.root, extras=[".git/"])

    def __iter__(self):
        paths = self.get_paths()
        for path in paths:
            yield SourceFile(self.root, path, self.url_base)

    def get_paths(self):
        for dirpath, dirnames, filenames in os.walk(self.root):
            for filename in filenames:
                path = os.path.relpath(os.path.join(dirpath, filename), self.root)
                if self.path_filter(path):
                    yield path

            dirnames[:] = [item for item in dirnames if self.path_filter(
                           os.path.relpath(os.path.join(dirpath, item), self.root) + "/")]
