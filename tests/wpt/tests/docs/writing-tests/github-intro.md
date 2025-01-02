# Introduction to GitHub

All the basics that you need to know are documented on this page, but for the
full GitHub documentation, visit [help.github.com][help].

If you are already an experienced Git/GitHub user, all you need to know is that
we use the [normal GitHub Pull Request workflow][github flow] for test
submissions.

If you are a first-time GitHub user, read on for more details of the workflow.

## Setup

1.  Create a GitHub account if you do not already have one on
    [github.com][github].

2.  Download and install the latest version of Git:
    [https://git-scm.com/downloads][git]; please refer to the instructions there
    for different platforms.

3.  Configure your settings so your commits are properly labeled:

    On Mac or Linux or Solaris, open the Terminal.

    On Windows, open Git Bash (From the Start Menu > Git > Git Bash).

    At the prompt, type:

        $ git config --global user.name "Your Name"

    _This will be the name that is displayed with your test submissions_

    Next, type:

        $ git config --global user.email "your_email@address.com"

    _This should be the email address you used to create the account in Step 1._

4.  (Optional) If you don't want to enter your username and password every
    time you talk to the remote server, you'll need to set up password caching.
    See [Caching your GitHub password in Git][password-caching].

## Fork the test repository

Now that you have Git set up, you will need to "fork" the test repository. Your
fork will be a completely independent version of the repository, hosted on
GitHub.com. This will enable you to [submit](#submit) your tests using a pull
request (more on this [below](#submit)).

1.  In the browser, go to [web-platform-tests on GitHub][main-repo].

2.  Click the ![fork](/assets/forkbtn.png) button in the upper right.

3.  The fork will take several seconds, then you will be redirected to your
    GitHub page for this forked repository.
    You will now be at
    **https://github.com/username/wpt**.

4. After the fork is complete, you're ready to [clone](#clone).

## Clone

If your [fork](#fork) was successful, the next step is to clone (download a copy of the files).

### Clone the test repository

Open a command prompt in the directory where you want to keep the tests. Then
execute the following command:

    $ git clone https://github.com/username/wpt.git

This will download the tests into a directory named for the repository: `wpt/`.

You should now have a full copy of the test repository on your local
machine. Feel free to browse the directories on your hard drive. You can also
[browse them on github.com][main-repo] and see the full history of
contributions there.

## Configure Remote / Upstream

Your forked repository is completely independent of the canonical repository,
which is commonly referred to as the "upstream" repository. Synchronizing your
forked repository with the upstream repository will keep your forked local copy
up-to-date with the latest commits.

In the vast majority of cases, the **only** upstream branch that you should
need to care about is `master`. If you see other branches in the repository,
you can generally safely ignore them.

1.  On the command line, navigate to to the directory where your forked copy of
    the repository is located.

2.  Make sure that you are on the master branch.  This will  be the case if you
    just forked, otherwise switch to master.

        $ git checkout master

3.  Next, add the remote of the repository your forked.  This assigns the
    original repository to a remote called "upstream":

        $ git remote add upstream https://github.com/web-platform-tests/wpt.git

4.  To pull in changes in the original repository that are not present in your
    local repository first fetch them:

        $ git fetch -p upstream

    Then merge them into your local repository:

        $ git merge upstream/master

    We recommend using `-p` to "prune" the outdated branches that would
    otherwise accumulate in your local repository.

For additional information, please see the [GitHub docs][github-fork-docs].

## Configure your environment

If all you intend to do is to load [manual tests](../writing-tests/manual) or [reftests](../writing-tests/reftests) from your local file system,
the above setup should be sufficient.
But many tests (and in particular, all [testharness.js tests](../writing-tests/testharness)) require a local web server.

See [Local Setup][local-setup] for more information.

## Branch

Now that you have everything locally, create a branch for your tests.

_Note: If you have already been through these steps and created a branch
and now want to create another branch, you should always do so from the
master branch. To do this follow the steps from the beginning of the [previous
section](#configure-remote-upstream). If you don't start with a clean master
branch you will end up with a big nested mess._

At the command line:

    $ git checkout -b topic

This will create a branch named `topic` and immediately
switch this to be your active working branch.

The branch name should describe specifically what you are testing. For example:

    $ git checkout -b flexbox-flex-direction-prop

You're ready to start writing tests! Come back to this page you're ready to
[commit](#commit) them or [submit](#submit) them for review.


## Commit

Before you submit your tests for review and contribution to the main test
repository, you'll need to first commit them locally, where you now have your
own personal version control system with git. In fact, as you are writing your
tests, you may want to save versions of your work as you go before you submit
them to be reviewed and merged.

1.  When you're ready to save a version of your work, open a command
    prompt and change to the directory where your files are.

2.  First, ask git what new or modified files you have:

        $ git status

    _This will show you files that have been added or modified_.

3.  For all new or modified files, you need to tell git to add them to the
    list of things you'd like to commit:

        $ git add [file1] [file2] ... [fileN]

    Or:

        $ git add [directory_of_files]

4.  Run `git status` again to see what you have on the 'Changes to be
    committed' list. These files are now 'staged'. Alternatively, you can run
    `git diff --staged` to see a visual representation of the changes to be
    committed.

5.  Once you've added everything, you can commit and add a message to this
    set of changes:

        $ git commit -m "Tests for indexed getters in the HTMLExampleInterface"

6.  Repeat these steps as many times as you'd like before you submit.

## Verify

The Web Platform Test project has an automated tool
to verify that coding conventions have been followed,
and to catch a number of common mistakes.

We recommend running this tool locally. That will help you discover and fix
issues that would make it hard for us to accept your contribution.

1. On the command line, navigate to to the directory where your clone
of the repository is located.

2. Run `./wpt lint`

3. Fix any mistake it reports and [commit](#commit) again.

For more details, see the [documentation about the lint tool](../writing-tests/lint-tool).

## Submit

If you're here now looking for more instructions, that means you've written
some awesome tests and are ready to submit them. Congratulations and welcome
back!

1.  The first thing you do before submitting them to the web-platform-tests
    repository is to push them back up to your fork:

        $ git push origin topic

    _Note: Here,_ `origin` _refers to remote repository from which you cloned
    (downloaded) the files after you forked, referred to as
    web-platform-tests.git in the previous example;_
    `topic` _refers to the name of your local branch that
    you want to share_.

2.  Now you can send a message that you have changes or additions you'd like
    to be reviewed and merged into the main (original) test repository. You do
    this by creating a pull request. In a browser, open the GitHub page for
    your forked repository: **https://github.com/username/wpt**.

3. Now create the pull request.  There are several ways to create a PR in the
GitHub UI.  Below is one method and others can be found on
[GitHub.com][github-createpr]

    1. Click the ![new pull request](../assets/pullrequestbtn.png) button.

    2.  On the left, you should see the base repository is the
        web-platform-tests/wpt. On the right, you should see your fork of that
        repository. In the branch menu of your forked repository, switch to `topic`

        If you see "There isn't anything to compare", make sure your fork and
        your `topic` branch is selected on the right side.

    3. Select the ![create pull request](../assets/createpr.png) button at the top.

    4. Scroll down and review the summary of changes.

    5. Scroll back up and in the Title field, enter a brief description for
       your submission.

       Example: "Tests for CSS Transforms skew() function."

    6.  If you'd like to add more detailed comments, use the comment field
    below.

    7.  Click ![the create pull request button](../assets/createpr.png)


4. Wait for feedback on your pull request and once your pull request is
accepted, delete your branch (see '[When Pull Request is Accepted](#cleanup)').

[This page on the submissions process](submission-process) has more detail
about what to expect when contributing code to WPT.

## Refine

Once you submit your pull request, a reviewer will check your proposed changes
for correctness and style. They may ask you to modify your code. When you are
ready to make the changes, follow these steps:

1.  Check out the branch corresponding to your changes e.g. if your branch was
    called `topic`
    run:

        $ git checkout topic

2.  Make the changes needed to address the comments, and commit them just like
    before.

3.  Push the changes to the remote branch containing the pull request:

        $ git push origin topic

4.  The pull request will automatically be updated with the new commit.

Sometimes it takes multiple iterations through a review before the changes are
finally accepted. Don't worry about this; it's totally normal. The goal of test
review is to work together to create the best possible set of tests for the web
platform.

## Cleanup
Once your pull request has been accepted, you will be notified in the GitHub
user interface, and you may get an email. At this point, your changes have been merged
into the main test repository. You do not need to take any further action
on the test but you should delete your branch. This can easily be done in
the GitHub user interface by navigating to the pull request and clicking the
"Delete Branch" button.

![pull request accepted delete branch](/assets/praccepteddelete.png)

Alternatively, you can delete the branch on the command line.

    $ git push origin --delete <branchName>

## Further Reading

Git is a very powerful tool, and there are many ways to achieve subtly
different results. Recognizing when (and understanding how) to use other
approaches is beyond the scope of this tutorial. [The Pro Git Book][git-book]
is a free digital resource that can help you learn more.

[local-setup]: ../running-tests/from-local-system
[git]: https://git-scm.com/downloads
[git-book]: https://git-scm.com/book
[github]: https://github.com/
[github-fork-docs]: https://help.github.com/articles/fork-a-repo
[github-createpr]: https://help.github.com/articles/creating-a-pull-request
[help]: https://help.github.com/
[main-repo]: https://github.com/web-platform-tests/wpt
[password-caching]: https://help.github.com/articles/caching-your-github-password-in-git
[github flow]: https://guides.github.com/introduction/flow/
