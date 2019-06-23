# Project Administration

This section documents all the information necessary to administer the
infrastructure which makes the project possible.

## Tooling

```eval_rst
.. toctree::
   :titlesonly:

   ../README
   ../tools/wptserve/docs/index.rst

.. toctree::
   :hidden:

   ../tools/wptserve/README
```

## Secrets

Some aspects of the infrastructure are only accessible to administrators.

```eval_rst
=========================  =========================  =========================
Project                    Secret                     Owners
=========================  =========================  =========================
[results-collection]       root SSH keys              boaz@bocoup.com, mike@bocoup.com, rick@bocoup.com
[results-collection]       Password for app secrets   boaz@bocoup.com, mike@bocoup.com, rick@bocoup.com
=========================  =========================  =========================

```

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
  - foolip@google.com
  - jeffcarp@google.com
  - lukebjerring@google.com
  - mike@bocoup.com
- [GitHub](https://github.com/): web-platform-tests
  - [@foolip](https://github.com/foolip)
  - [@Hexcles](https://github.com/Hexcles)
  - [@jgraham](https://github.com/jgraham)
  - [@plehegar](https://github.com/plehegar)
  - [@thejohnjansen](https://github.com/thejohnjansen)
  - [@youennf](https://github.com/youennf)
  - [@zcorpan](https://github.com/zcorpan)
- [GitHub](https://github.com/): w3c
  - [@plehegar](https://github.com/plehegar)
  - [@sideshowbarker](https://github.com/sideshowbarker)
- [Google Cloud Platform](https://cloud.google.com/): wptdashboard
  - boaz@bocoup.com
  - foolip@google.com
  - geoffers@gmail.com
  - jeffcarp@google.com
  - kereliuk@google.com
  - lukebjerring@google.com
  - markdittmer@google.com
  - mike@bocoup.com
  - rick@bocoup.com
- [Amazon AWS](https://aws.amazon.com/): results-collection infrastructure
  - boaz@bocoup.com
  - mike@bocoup.com
  - rick@bocoup.com
- E-mail address: wpt.pr.bot@gmail.com
  - boaz@bocoup.com
  - mike@bocoup.com
  - simon@bocoup.com
- [Heroku](https://heroku.com/): wpt.pr.bot@gmail.com
  - boaz@bocoup.com
  - mike@bocoup.com
  - simon@bocoup.com
- [GitHub](https://github.com/): @wpt-pr-bot account
  - boaz@bocoup.com
  - mike@bocoup.com
  - simon@bocoup.com

[results-collection]: https://github.com/web-platform-tests/results-collection
[web-platform-tests]: https://github.com/e3c/web-platform-tests
[wpt.fyi]: https://github.com/web-platform-tests/wpt.fyi
