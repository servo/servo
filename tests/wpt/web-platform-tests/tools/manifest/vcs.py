import os
import subprocess

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
            return subprocess.check_output(full_cmd, cwd=repo_path, stderr=subprocess.STDOUT)
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
        cmd = ["ls-tree", "-r", "-z", "--name-only", "HEAD"]
        local_changes = self._local_changes()
        for rel_path in self.git(*cmd).split("\0")[:-1]:
            if not os.path.isdir(os.path.join(self.root, rel_path)):
                if rel_path in local_changes:
                    contents = self._show_file(rel_path)
                else:
                    contents = None
                yield SourceFile(self.root,
                                 rel_path,
                                 self.url_base,
                                 contents=contents)


class FileSystem(object):
    def __init__(self, root, url_base):
        self.root = root
        self.url_base = url_base
        from gitignore import gitignore
        self.path_filter = gitignore.PathFilter(self.root)

    def __iter__(self):
        is_root = True
        for dir_path, dir_names, filenames in os.walk(self.root):
            rel_root = os.path.relpath(dir_path, self.root)

            if is_root:
                dir_names[:] = [item for item in dir_names if item not in
                                ["tools", "resources", ".git"]]
                is_root = False

            for filename in filenames:
                rel_path = os.path.join(rel_root, filename)
                if self.path_filter(rel_path):
                    yield SourceFile(self.root,
                                     rel_path,
                                     self.url_base)
