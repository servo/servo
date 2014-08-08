set -e
export DISPLAY=:1.0
export RUST_TEST_TASKS=1
case $1 in
content)
  make check-content
;;
ref)
  make check-ref-cpu
;;
unit-doc)
  make check-servo

  mv x86_64-unknown-linux-gnu/rust_snapshot/rust-*/doc .
  cp ../src/etc/doc.servo.org/* doc
  make doc

  if [ $TRAVIS_BRANCH = master ] && [ $TRAVIS_PULL_REQUEST = false ]
  then
      echo '<meta http-equiv=refresh content=0;url=servo/index.html>' > doc/index.html
      sudo pip install ghp-import
      ghp-import -n doc
      git push -fq https://${TOKEN}@github.com/${TRAVIS_REPO_SLUG}.git gh-pages
  fi
;;
*) echo "Task $1 not enabled for Linux"
esac
