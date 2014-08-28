set -e
export DISPLAY=:1.0
export RUST_TEST_TASKS=1
case $1 in
content)
  ./mach test-content
;;
ref)
  ./mach test-ref --kind cpu
;;
unit-doc)
  ./mach test-unit

  ./mach doc
  cp etc/doc.servo.org/* target/doc


  if [ $TRAVIS_BRANCH = master ] && [ $TRAVIS_PULL_REQUEST = false ]
  then
      echo '<meta http-equiv=refresh content=0;url=servo/index.html>' > target/doc/index.html
      sudo pip install ghp-import
      ghp-import -n target/doc
      git push -qf https://${TOKEN}@github.com/servo/doc.servo.org.git gh-pages
  fi
;;
*) echo "Task $1 not enabled for Linux"
esac
