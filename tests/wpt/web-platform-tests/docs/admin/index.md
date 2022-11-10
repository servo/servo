# Project Administration

This section documents all the information necessary to administer the
infrastructure which makes the project possible.

## Tooling

```eval_rst
.. toctree::
   :titlesonly:

   ../README
   /tools/wptrunner/README.rst
   /tools/wptserve/docs/index.rst
   pywebsocket3

.. toctree::
   :hidden:

   ../tools/wptserve/README
   ../tools/third_party/pywebsocket3/README
```

### Indices and tables

```eval_rst
* :ref:`modindex`
* :ref:`genindex`
* :ref:`search`
```

## Secrets

SSL certificates for all HTTPS-enabled domains are retrieved via [Let's
Encrypt](https://letsencrypt.org/), so that data does not represent an
explicitly-managed secret.

## Third-party account owners

- (unknown registrar): https://web-platform-tests.org
  - jgraham@hoppipolla.co.uk
- (unknown registrar): https://w3c-test.org
  - mike@w3.org
- (unknown registrar): http://testthewebforward.org
  - web-human@w3.org
- [Google Domains](https://domains.google/): https://wpt.fyi
  - danielrsmith@google.com
  - foolip@google.com
  - kyleju@google.com
  - pastithas@google.com
- [GitHub](https://github.com/): web-platform-tests
  - [@foolip](https://github.com/foolip)
  - [@jgraham](https://github.com/jgraham)
  - [@plehegar](https://github.com/plehegar)
  - [@thejohnjansen](https://github.com/thejohnjansen)
  - [@youennf](https://github.com/youennf)
  - [@zcorpan](https://github.com/zcorpan)
- [GitHub](https://github.com/): w3c
  - [@plehegar](https://github.com/plehegar)
  - [@sideshowbarker](https://github.com/sideshowbarker)
- [Google Cloud Platform](https://cloud.google.com/): wptdashboard{-staging}
  - danielrsmith@google.com
  - foolip@google.com
  - kyleju@google.com
  - pastithas@google.com
- [Google Cloud Platform](https://cloud.google.com/): wpt-live
  - danielrsmith@chromium.org
  - foolip@chromium.org
  - kyleju@chromium.org
  - mike@bocoup.com
  - pastithas@chromium.org
  - The DNS for wpt.live, not-wpt.live, wptpr.live, and not-wptpr.live are also managed in this project, while the domains are registered with a Google-internal mechanism.
- [Google Cloud Platform](https://cloud.google.com/): wpt-pr-bot
  - danielrsmith@google.com
  - foolip@google.com
  - kyleju@google.com
  - pastithas@google.com
- E-mail address: wpt.pr.bot@gmail.com
  - smcgruer@google.com
  - boaz@bocoup.com
  - mike@bocoup.com
  - simon@bocoup.com
- [GitHub](https://github.com/): @wpt-pr-bot account
  - smcgruer@google.com
  - boaz@bocoup.com
  - mike@bocoup.com
  - simon@bocoup.com

## Emergency playbook

### Lock down write access to the repo

**Recommended but not yet verified approach:** Create a [new branch protection
rule](https://github.com/web-platform-tests/wpt/settings/branch_protection_rules/new)
that applies to `*` (i.e. all branches), and check "Restrict who can push to
matching branches". This should prevent everyone except those with the
"Maintain" role (currently only the GitHub admins listed above) from pushing
to *any* branch. To lift the limit, delete this branch protection rule.

**Alternative approach proven to work in
[#21424](https://github.com/web-platform-tests/wpt/issues/21424):** Go to
[manage access](https://github.com/web-platform-tests/wpt/settings/access),
and change the permission of "reviewers" to "Read". To lift the limit, change
it back to "Write". This has the known downside of *resubscribing all reviewers
to repo notifications*.
