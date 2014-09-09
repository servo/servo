#!/bin/bash

set -e

build_docs() {
    ./mach doc
    cp etc/doc.servo.org/* target/doc/
    cp -R rust/doc/* target/doc/

    if [ "${TRAVIS_BRANCH}" = "master" ] && [ "${TRAVIS_PULL_REQUEST}" = "false" ]; then
        echo '<meta http-equiv=refresh content=0;url=servo/index.html>' > target/doc/index.html
        sudo pip install ghp-import
        ghp-import -n target/doc
        git push -qf https://${TOKEN}@github.com/servo/doc.servo.org.git gh-pages
    fi
}

build_servo() {
    ./mach build -j 2
}

build_cef() {
    ./mach build-cef -j 2
}

IFS="," read -ra tasks <<< "${TASKS}"
for t in "${tasks[@]}"; do
    # OS specific setup
    case ${TRAVIS_OS_NAME} in
        linux)
            export DISPLAY=:1.0
            export RUST_TEST_TASKS=1
            ;;
        osx)
            ;;
    esac

    case $t in
        build)
            ./mach build -j 2
            ;;
        build-cef)
            ./mach build-cef -j 2
            ;;
        test-content)
            ./mach test-content
            ;;
        test-ref)
            ./mach test-ref --kind cpu
            ;;
        test-wpt1)
            ./mach test-wpt --processes=2 --total-chunks=2 --this-chunk=1
            ;;
        test-wpt1)
            ./mach test-wpt --processes=2 --total-chunks=2 --this-chunk=2
            ;;
        *)
            echo "Task $t not recognized."
            ;;
    esac
done
