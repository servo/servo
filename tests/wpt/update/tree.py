# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

import re
import tempfile

from wptrunner import update as wptupdate

from wptrunner.update.tree import Commit, CommitMessage, get_unique_name

class HgTree(wptupdate.tree.HgTree):
    def __init__(self, *args, **kwargs):
        self.commit_cls = kwargs.pop("commit_cls", Commit)
        wptupdate.tree.HgTree.__init__(self, *args, **kwargs)

    # TODO: The extra methods for upstreaming patches from a
    # hg checkout

class GitTree(wptupdate.tree.GitTree):
    def __init__(self, *args, **kwargs):
        """Extension of the basic GitTree with extra methods for
        transfering patches"""
        commit_cls = kwargs.pop("commit_cls", Commit)
        wptupdate.tree.GitTree.__init__(self, *args, **kwargs)
        self.commit_cls = commit_cls

    def create_branch(self, name, ref=None):
        """Create a named branch,

        :param name: String representing the branch name.
        :param ref: None to use current HEAD or rev that the branch should point to"""

        args = []
        if ref is not None:
            if hasattr(ref, "sha1"):
                ref = ref.sha1
            args.append(ref)
        self.git("branch", name, *args)

    def commits_by_message(self, message, path=None):
        """List of commits with messages containing a given string.

        :param message: The string that must be contained in the message.
        :param path: Path to a file or directory the commit touches
        """
        args = ["--pretty=format:%H", "--reverse", "-z", "--grep=%s" % message]
        if path is not None:
            args.append("--")
            args.append(path)
        data = self.git("log", *args)
        return [self.commit_cls(self, sha1) for sha1 in data.split("\0")]

    def log(self, base_commit=None, path=None):
        """List commits touching a certian path from a given base commit.

        :base_param commit: Commit object for the base commit from which to log
        :param path: Path that the commits must touch
        """
        args = ["--pretty=format:%H", "--reverse", "-z"]
        if base_commit is not None:
            args.append("%s.." % base_commit.sha1)
        if path is not None:
            args.append("--")
            args.append(path)
        data = self.git("log", *args)
        return [self.commit_cls(self, sha1) for sha1 in data.split("\0") if sha1]

    def import_patch(self, patch):
        """Import a patch file into the tree and commit it

        :param patch: a Patch object containing the patch to import
        """

        with tempfile.NamedTemporaryFile() as f:
            f.write(patch.diff)
            f.flush()
            f.seek(0)
            self.git("apply", "--index", f.name)
        self.git("commit", "-m", patch.message.text, "--author=%s" % patch.full_author)

    def rebase(self, ref, continue_rebase=False):
        """Rebase the current branch onto another commit.

        :param ref: A Commit object for the commit to rebase onto
        :param continue_rebase: Continue an in-progress rebase"""
        if continue_rebase:
            args = ["--continue"]
        else:
            if hasattr(ref, "sha1"):
                ref = ref.sha1
            args = [ref]
        self.git("rebase", *args)

    def push(self, remote, local_ref, remote_ref, force=False):
        """Push local changes to a remote.

        :param remote: URL of the remote to push to
        :param local_ref: Local branch to push
        :param remote_ref: Name of the remote branch to push to
        :param force: Do a force push
        """
        args = []
        if force:
            args.append("-f")
        args.extend([remote, "%s:%s" % (local_ref, remote_ref)])
        self.git("push", *args)

    def unique_branch_name(self, prefix):
        """Get an unused branch name in the local tree

        :param prefix: Prefix to use at the start of the branch name"""
        branches = [ref[len("refs/heads/"):] for sha1, ref in self.list_refs()
                    if ref.startswith("refs/heads/")]
        return get_unique_name(branches, prefix)

class Patch(object):
    def __init__(self, author, email, message, diff):
        self.author = author
        self.email = email
        if isinstance(message, CommitMessage):
            self.message = message
        else:
            self.message = GeckoCommitMessage(message)
        self.diff = diff

    def __repr__(self):
        return "<Patch (%s)>" % self.message.full_summary

    @property
    def full_author(self):
        return "%s <%s>" % (self.author, self.email)

    @property
    def empty(self):
        return bool(self.diff.strip())


class GeckoCommitMessage(CommitMessage):
    """Commit message following the Gecko conventions for identifying bug number
    and reviewer"""

    # c.f. http://hg.mozilla.org/hgcustom/version-control-tools/file/tip/hghooks/mozhghooks/commit-message.py
    # which has the regexps that are actually enforced by the VCS hooks. These are
    # slightly different because we need to parse out specific parts of the message rather
    # than just enforce a general pattern.

    _bug_re = re.compile("^Bug (\d+)[^\w]*(?:Part \d+[^\w]*)?(.*?)\s*(?:r=(\w*))?$",
                         re.IGNORECASE)

    _backout_re = re.compile("^(?:Back(?:ing|ed)\s+out)|Backout|(?:Revert|(?:ed|ing))",
                             re.IGNORECASE)
    _backout_sha1_re = re.compile("(?:\s|\:)(0-9a-f){12}")

    def _parse_message(self):
        CommitMessage._parse_message(self)

        if self._backout_re.match(self.full_summary):
            self.backouts = self._backout_re.findall(self.full_summary)
        else:
            self.backouts = []

        m = self._bug_re.match(self.full_summary)
        if m is not None:
            self.bug, self.summary, self.reviewer = m.groups()
        else:
            self.bug, self.summary, self.reviewer = None, self.full_summary, None


class GeckoCommit(Commit):
    msg_cls = GeckoCommitMessage

    def export_patch(self, path=None):
        """Convert a commit in the tree to a Patch with the bug number and
        reviewer stripped from the message"""
        args = ["%s^..%s" % (self.sha1, self.sha1)]
        if path is not None:
            args.append("--")
            args.append(path)

        diff = self.git("diff", *args)

        return Patch(self.author, self.email, self.message, diff)

