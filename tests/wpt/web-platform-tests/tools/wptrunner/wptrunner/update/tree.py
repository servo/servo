import os
import re
import subprocess

from .. import vcs
from ..vcs import git, hg


def get_unique_name(existing, initial):
    """Get a name either equal to initial or of the form initial_N, for some
    integer N, that is not in the set existing.


    :param existing: Set of names that must not be chosen.
    :param initial: Name, or name prefix, to use"""
    if initial not in existing:
        return initial
    for i in xrange(len(existing) + 1):
        test = "%s_%s" % (initial, i + 1)
        if test not in existing:
            return test
    assert False

class NoVCSTree(object):
    name = "non-vcs"

    def __init__(self, root=None):
        if root is None:
            root = os.path.abspath(os.curdir)
        self.root = root

    @classmethod
    def is_type(cls, path=None):
        return True

    @property
    def is_clean(self):
        return True

    def add_new(self, prefix=None):
        pass

    def create_patch(self, patch_name, message):
        pass

    def update_patch(self, include=None):
        pass

    def commit_patch(self):
        pass


class HgTree(object):
    name = "mercurial"

    def __init__(self, root=None):
        if root is None:
            root = hg("root").strip()
        self.root = root
        self.hg = vcs.bind_to_repo(hg, self.root)

    def __getstate__(self):
        rv = self.__dict__.copy()
        del rv['hg']
        return rv

    def __setstate__(self, dict):
        self.__dict__.update(dict)
        self.hg = vcs.bind_to_repo(vcs.hg, self.root)

    @classmethod
    def is_type(cls, path=None):
        kwargs = {"log_error": False}
        if path is not None:
            kwargs["repo"] = path
        try:
            hg("root", **kwargs)
        except Exception:
            return False
        return True

    @property
    def is_clean(self):
        return self.hg("status").strip() == ""

    def add_new(self, prefix=None):
        if prefix is not None:
            args = ("-I", prefix)
        else:
            args = ()
        self.hg("add", *args)

    def create_patch(self, patch_name, message):
        try:
            self.hg("qinit", log_error=False)
        except subprocess.CalledProcessError:
            pass

        patch_names = [item.strip() for item in self.hg("qseries").split("\n") if item.strip()]

        suffix = 0
        test_name = patch_name
        while test_name in patch_names:
            suffix += 1
            test_name = "%s-%i" % (patch_name, suffix)

        self.hg("qnew", test_name, "-X", self.root, "-m", message)

    def update_patch(self, include=None):
        if include is not None:
            args = []
            for item in include:
                args.extend(["-I", item])
        else:
            args = ()

        self.hg("qrefresh", *args)
        return True

    def commit_patch(self):
        self.hg("qfinish")

    def contains_commit(self, commit):
        try:
            self.hg("identify", "-r", commit.sha1)
            return True
        except subprocess.CalledProcessError:
            return False


class GitTree(object):
    name = "git"

    def __init__(self, root=None, log_error=True):
        if root is None:
            root = git("rev-parse", "--show-toplevel", log_error=log_error).strip()
        self.root = root
        self.git = vcs.bind_to_repo(git, self.root, log_error=log_error)
        self.message = None
        self.commit_cls = Commit

    def __getstate__(self):
        rv = self.__dict__.copy()
        del rv['git']
        return rv

    def __setstate__(self, dict):
        self.__dict__.update(dict)
        self.git = vcs.bind_to_repo(vcs.git, self.root)

    @classmethod
    def is_type(cls, path=None):
        kwargs = {"log_error": False}
        if path is not None:
            kwargs["repo"] = path
        try:
            git("rev-parse", "--show-toplevel", **kwargs)
        except Exception:
            return False
        return True

    @property
    def rev(self):
        """Current HEAD revision"""
        if vcs.is_git_root(self.root):
            return self.git("rev-parse", "HEAD").strip()
        else:
            return None

    @property
    def is_clean(self):
        return self.git("status").strip() == ""

    def add_new(self, prefix=None):
        """Add files to the staging area.

        :param prefix: None to include all files or a path prefix to
                       add all files under that path.
        """
        if prefix is None:
            args = ("-a",)
        else:
            args = ("--no-ignore-removal", prefix)
        self.git("add", *args)

    def list_refs(self, ref_filter=None):
        """Get a list of sha1, name tuples for references in a repository.

        :param ref_filter: Pattern that reference name must match (from the end,
                           matching whole /-delimited segments only
        """
        args = []
        if ref_filter is not None:
            args.append(ref_filter)
        data = self.git("show-ref", *args)
        rv = []
        for line in data.split("\n"):
            if not line.strip():
                continue
            sha1, ref = line.split()
            rv.append((sha1, ref))
        return rv

    def list_remote(self, remote, ref_filter=None):
        """Return a list of (sha1, name) tupes for references in a remote.

        :param remote: URL of the remote to list.
        :param ref_filter: Pattern that the reference name must match.
        """
        args = []
        if ref_filter is not None:
            args.append(ref_filter)
        data = self.git("ls-remote", remote, *args)
        rv = []
        for line in data.split("\n"):
            if not line.strip():
                continue
            sha1, ref = line.split()
            rv.append((sha1, ref))
        return rv

    def get_remote_sha1(self, remote, branch):
        """Return the SHA1 of a particular branch in a remote.

        :param remote: the remote URL
        :param branch: the branch name"""
        for sha1, ref in self.list_remote(remote, branch):
            if ref == "refs/heads/%s" % branch:
                return self.commit_cls(self, sha1)
        assert False

    def create_patch(self, patch_name, message):
        # In git a patch is actually a commit
        self.message = message

    def update_patch(self, include=None):
        """Commit the staged changes, or changes to listed files.

        :param include: Either None, to commit staged changes, or a list
                        of filenames (which must already be in the repo)
                        to commit
        """
        if include is not None:
            args = tuple(include)
        else:
            args = ()

        if self.git("status", "-uno", "-z", *args).strip():
            self.git("add", *args)
            return True
        return False

    def commit_patch(self):
        assert self.message is not None

        if self.git("diff", "--name-only", "--staged", "-z").strip():
            self.git("commit", "-m", self.message)
            return True

        return False

    def init(self):
        self.git("init")
        assert vcs.is_git_root(self.root)

    def checkout(self, rev, branch=None, force=False):
        """Checkout a particular revision, optionally into a named branch.

        :param rev: Revision identifier (e.g. SHA1) to checkout
        :param branch: Branch name to use
        :param force: Force-checkout
        """
        assert rev is not None

        args = []
        if branch:
            branches = [ref[len("refs/heads/"):] for sha1, ref in self.list_refs()
                        if ref.startswith("refs/heads/")]
            branch = get_unique_name(branches, branch)

            args += ["-b", branch]

        if force:
            args.append("-f")
        args.append(rev)
        self.git("checkout", *args)

    def update(self, remote, remote_branch, local_branch):
        """Fetch from the remote and checkout into a local branch.

        :param remote: URL to the remote repository
        :param remote_branch: Branch on the remote repository to check out
        :param local_branch: Local branch name to check out into
        """
        if not vcs.is_git_root(self.root):
            self.init()
        self.git("clean", "-xdf")
        self.git("fetch", remote, "%s:%s" % (remote_branch, local_branch))
        self.checkout(local_branch)
        self.git("submodule", "update", "--init", "--recursive")

    def clean(self):
        self.git("checkout", self.rev)
        self.git("branch", "-D", self.local_branch)

    def paths(self):
        """List paths in the tree"""
        repo_paths = [self.root] + [os.path.join(self.root, path)
                                    for path in self.submodules()]

        rv = []

        for repo_path in repo_paths:
            paths = vcs.git("ls-tree", "-r", "--name-only", "HEAD", repo=repo_path).split("\n")
            rv.extend(os.path.relpath(os.path.join(repo_path, item), self.root) for item in paths
                      if item.strip())
        return rv

    def submodules(self):
        """List submodule directories"""
        output = self.git("submodule", "status", "--recursive")
        rv = []
        for line in output.split("\n"):
            line = line.strip()
            if not line:
                continue
            parts = line.split(" ")
            rv.append(parts[1])
        return rv

    def contains_commit(self, commit):
        try:
            self.git("rev-parse", "--verify", commit.sha1)
            return True
        except subprocess.CalledProcessError:
            return False


class CommitMessage(object):
    def __init__(self, text):
        self.text = text
        self._parse_message()

    def __str__(self):
        return self.text

    def _parse_message(self):
        lines = self.text.splitlines()
        self.full_summary = lines[0]
        self.body = "\n".join(lines[1:])


class Commit(object):
    msg_cls = CommitMessage

    _sha1_re = re.compile("^[0-9a-f]{40}$")

    def __init__(self, tree, sha1):
        """Object representing a commit in a specific GitTree.

        :param tree: GitTree to which this commit belongs.
        :param sha1: Full sha1 string for the commit
        """
        assert self._sha1_re.match(sha1)

        self.tree = tree
        self.git = tree.git
        self.sha1 = sha1
        self.author, self.email, self.message = self._get_meta()

    def __getstate__(self):
        rv = self.__dict__.copy()
        del rv['git']
        return rv

    def __setstate__(self, dict):
        self.__dict__.update(dict)
        self.git = self.tree.git

    def _get_meta(self):
        author, email, message = self.git("show", "-s", "--format=format:%an\n%ae\n%B", self.sha1).split("\n", 2)
        return author, email, self.msg_cls(message)
