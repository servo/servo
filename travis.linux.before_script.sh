/usr/bin/Xorg :1 -noreset +extension GLX +extension RANDR +extension RENDER -logfile ./xorg.log -config ./src/test/ci/xorg.conf &

# Patch the broken font config files on ubuntu 12.04 lts - this should be removed when travis moves to ubuntu 14.04 lts
sudo cp src/test/ci/fontconfig/* /etc/fonts/conf.avail/
