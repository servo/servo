## The Servo Parallel Browser Project

It builds on Linux and OS X and requires the newest rustc you can find

    git clone git://github.com/mozilla/servo.git
    cd servo
    git submodule init
    git submodule update
    mkdir build && cd build
    ../configure
    make check && make
    ./servo ../test.html
