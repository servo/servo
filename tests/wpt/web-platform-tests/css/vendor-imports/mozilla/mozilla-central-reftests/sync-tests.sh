#!/bin/bash

if [ "x$1" != "x" ]; then
  MOZTREE="$1"
else
  MOZTREE="$HOME/builds/clean-mozilla-central/mozilla/"
fi

cd "$(dirname "$0")"

if [ "$(git status -s . | wc -l)" != "0" ]
then
    echo "Directory not clean" 1>&2
    exit 1
fi

if [ -e "$MOZTREE/.git" ]
then
  MOZREV="$(cd "$MOZTREE" && git cinnabar git2hg HEAD)"
else
  MOZREV="$(cd "$MOZTREE" && hg par --temp='{node}')"
fi

rsync -avz --delete --filter=". ./sync-tests-filter" "$MOZTREE"/layout/reftests/w3c-css/submitted/ ./
sed -i -e 's/^\(\(fails\|needs-focus\|random\|skip\|asserts\|slow\|require-or\|silentfail\|pref\|test-pref\|ref-pref\|fuzzy\)[^ ]* *\?\)\+//;/^default-preferences /d;s/ \?# \?\(TC: \)\?[bB]ug.*//' $(find . -name reftest.list)
sed -i -e 's/-moz-crisp-edges/pixelated/g' $(find . -regex ".*\.\(xht\|xhtml\|html\|css\)")
git add -A .
git commit -m"Sync Mozilla CSS tests as of https://hg.mozilla.org/mozilla-central/rev/$MOZREV ." -e .
