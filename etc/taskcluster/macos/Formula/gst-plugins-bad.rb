class GstPluginsBad < Formula
  desc "GStreamer plugins less supported, not fully tested"
  homepage "https://gstreamer.freedesktop.org/"
  url "https://gstreamer.freedesktop.org/src/gst-plugins-bad/gst-plugins-bad-1.16.0.tar.xz"
  sha256 "22139de35626ada6090bdfa3423b27b7fc15a0198331d25c95e6b12cb1072b05"

  bottle do
    sha256 "7c27cd50644867490e5aa36860f3046889167234c4f139d56c895f9edd1c3a99" => :mojave
    sha256 "c7b7c9586a08c3f7a1c0677165ecb0a0e9a989516eb3effdbb74dd285100dff0" => :high_sierra
    sha256 "fc98180089cae089882fb91240280498ef33094f18b27b2cdfd9923a236e06de" => :sierra
  end

  head do
    url "https://anongit.freedesktop.org/git/gstreamer/gst-plugins-bad.git"

    depends_on "autoconf" => :build
    depends_on "automake" => :build
  end

  depends_on "gobject-introspection" => :build
  depends_on "libtool" => :build
  depends_on "pkg-config" => :build
  depends_on "faac"
  depends_on "faad2"
  depends_on "gettext"
  depends_on "gst-plugins-base"
  depends_on "jpeg"
  depends_on "libmms"
  depends_on "openssl"
  depends_on "opus"
  depends_on "orc"
  depends_on "libnice"
  depends_on "srtp"

  def install
    args = %W[
      --prefix=#{prefix}
      --disable-yadif
      --disable-examples
      --disable-debug
      --disable-dependency-tracking
      --enable-introspection=yes
    ]

    if build.head?
      # autogen is invoked in "stable" build because we patch configure.ac
      ENV["NOCONFIGURE"] = "yes"
      system "./autogen.sh"
    end

    system "./configure", *args
    system "make"
    system "make", "install"
  end

  test do
    gst = Formula["gstreamer"].opt_bin/"gst-inspect-1.0"
    output = shell_output("#{gst} --plugin dvbsuboverlay")
    assert_match version.to_s, output
  end
end
