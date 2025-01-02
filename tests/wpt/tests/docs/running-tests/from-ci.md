# Running Tests on CI

Contributors with write access to the repository can trigger full runs in the
same CI systems used to produce results for [wpt.fyi](https://wpt.fyi). The runs
are triggered by pushing to branch names on the form `triggers/$browser_$channel`
and the results will be automatically submitted to wpt.fyi.

This is useful when making infrastructure changes that could affect very many
tests, in order to avoid regressions.

Note: Full runs use a lot of CI resources, so please take care to not trigger
them more than necessary.

Instructions:

 * Base your changes on a commit for which there are already results in wpt.fyi.

 * Determine which branch name to push to by looking for `refs/heads/triggers/`
   in `.azure-pipelines.yml` and `.taskcluster.yml`. For example, to trigger a
   full run of Safari Technology Preview, the branch name is
   `triggers/safari_preview`.

 * Force push to the branch, for example:
   `git push --force-with-lease origin HEAD:triggers/safari_preview`.
   The `--force-with-lease` argument is to detect if someone else has just
   pushed. When this happens wait for the checkout step of their triggered run
   to finish before you force push again.

You can see if the run started from the commit status on GitHub's commits listing
([example](https://github.com/web-platform-tests/wpt/commits/triggers/safari_preview))
and if successful the results will show up on wpt.fyi within 10 minutes
([example](https://wpt.fyi/runs?product=safari)).
