#!/bin/bash

set -ex

neutral_status=0
source_revision=$(git rev-parse HEAD)
# The token available in the `GITHUB_TOKEN` variable may be used to push to the
# repository, but GitHub Pages will not rebuild the website in response to such
# events. Use an access token generated for the project's machine user,
# wpt-pr-bot.
#
# https://help.github.com/en/articles/generic-jekyll-build-failures
remote_url=https://${DEPLOY_TOKEN}@github.com/web-platform-tests/wpt.git

function json_property {
  cat ${1} | \
    python -c "import json, sys; print json.load(sys.stdin).get(\"${2}\", \"\")"
}

function is_pull_request {
  test -n "$(json_property ${GITHUB_EVENT_PATH} pull_request)"
}

function targets_master {
  test $(json_property ${GITHUB_EVENT_PATH} ref) == 'refs/heads/master'
}

git config --global user.email "wpt-pr-bot@users.noreply.github.com"
git config --global user.name "wpt-pr-bot"

# Prepare the output directory so that the new build can be pushed to the
# repository as an incremental change to the prior build.
mkdir -p docs/_build/html
cd docs/_build/html
git init
git fetch --depth 1 ${remote_url} gh-pages
git checkout FETCH_HEAD
git rm -rf .

# Build the website
cd ../..
pip install -r requirements.txt
make html

cd _build/html
# Configure DNS
echo web-platform-tests.org > CNAME
# Disable Jekyll
# https://github.blog/2009-12-29-bypassing-jekyll-on-github-pages/
touch .nojekyll

# Publish the website by pushing the built contents to the `gh-pages` branch
git add .

echo This submission alters the compiled files as follows

git diff --staged

if is_pull_request ; then
  echo Submission comes from a pull request. Exiting without publishing.

  exit ${neutral_status}
fi

if ! targets_master ; then
  echo Submission does not target the 'master' branch. Exiting without publishing.

  exit ${neutral_status}
fi

if git diff --exit-code --quiet --staged ; then
  echo No change to the website contents. Exiting without publishing.

  exit ${neutral_status}
fi

git commit --message "Build documentation

These files were generated from commit ${source_revision}"

git push --force ${remote_url} HEAD:gh-pages
