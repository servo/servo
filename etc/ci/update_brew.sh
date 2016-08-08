#!/usr/bin/env bash

set -o errexit
set -o nounset
set -o pipefail

SCRIPTDIR=$PWD/`dirname $0`
cd "$SCRIPTDIR/../.."

PACKAGEPATH=`ls -t target/servo-????-??-??.tar.gz | head -n 1`
PACKAGENAME=`basename $PACKAGEPATH`
VERSION=`echo $PACKAGENAME| sed -n "s/servo-.*\([0-9]\{4\}\)-\([0-9]\{2\}\)-\([0-9]\{2\}\).tar.gz*/\1.\2.\3/p"`
PACKAGEURL="http://people.mozilla.com/~prouget/graphene/brew/$PACKAGENAME" 
SHA=`shasum -a 256 $PACKAGEPATH | sed -e 's/ .*//'`

if [[ -z $VERSION ]]; then
  echo "Package doesn't havent the right format: $PACKAGENAME"
  exit 1
fi

scp $PACKAGEPATH prouget@people.mozilla.com:public_html/graphene/brew/

echo "Package successfuly uploaded to $PACKAGEURL"

tmp_dir=`mktemp -d -t homebrew-servo`
cd $tmp_dir
echo $tmp_dir

echo "Cloning"
git clone https://github.com/paulrouget/homebrew-servo
cd homebrew-servo


# Not using "/" as it's used in PACKAGEURL
cat $SCRIPTDIR/servo-binary-formula.rb.in | sed \
  "s|PACKAGEURL|$PACKAGEURL|g
   s|SHA|$SHA|g
   s|VERSION|$VERSION|g" > Formula/servo-bin.rb

git add ./Formula/servo-bin.rb
git commit -m "Version bump: $VERSION"

git push -qf "git@github.com:paulrouget/homebrew-servo.git" master
rm -rf $tmp_dir
