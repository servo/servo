wget https://github.com/ferjm/gstreamer-1.14.1-ubuntu-trusty/raw/master/gstreamer.tar.gz -O gstreamer.tar.gz
tar -zxvf gstreamer.tar.gz
rm gstreamer.tar.gz
sed -i "s;prefix=/root/gstreamer;prefix=$PWD/gstreamer;g" $PWD/gstreamer/lib/x86_64-linux-gnu/pkgconfig/*.pc
