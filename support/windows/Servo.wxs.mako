<?xml version="1.0" encoding="utf-8"?>
<Wix xmlns="http://schemas.microsoft.com/wix/2006/wi">
  <Product Name="Servo Tech Demo"
           Manufacturer="Mozilla Research"
           Id="5807391a-3a17-476b-a5d2-5f1912569762"
           UpgradeCode="060cd15d-eab1-4614-b438-3988e3efdcf1"
           Language="1033"
           Codepage="1252"
           Version="1.0.0">
    <Package Id="*"
             Keywords="Installer"
             Description="Servo Tech Demo Installer"
             Manufacturer="Mozilla Research"
             InstallerVersion="200"
             Platform="x64"
             Languages="1033"
             SummaryCodepage="1252"
             Compressed="yes"/>
    <Media Id="1"
           Cabinet="Servo.cab"
           EmbedCab="yes"/>
    <Directory Id="TARGETDIR" Name="SourceDir">
      <Directory Id="ProgramFiles64Folder" Name="PFiles">
        <Directory Id="MozResearch" Name="Mozilla Research">
          <Directory Id="INSTALLDIR" Name="Servo Tech Demo">
            <Component Id="Servo"
                       Guid="95bcea71-78bb-4ec8-9766-44bc01443840"
                       Win64="yes">
              <File Id="ServoEXE"
                    Name="servo.exe"
                    DiskId="1"
                    Source="${windowize(exe_path)}\servo.exe"
                    KeyPath="yes">
                <Shortcut Id="StartMenuServoTechDemo"
                          Directory="ProgramMenuDir"
                          Name="Servo Tech Demo"
                          WorkingDirectory="INSTALLDIR"
                          Icon="Servo.ico"
                          IconIndex="0"
                          Arguments="-w --pref dom.mozbrowser.enabled --pref shell.builtin-key-shortcuts.enabled=false browserhtml\index.html"
                          Advertise="yes"/>
              </File>
              <File Id="ServoManifest"
                    Name="servo.exe.manifest"
                    Source="${windowize(exe_path)}\servo.exe.manifest"
                    DiskId="1"/>

              <File Id="StdcxxDLL"
                    Name="libstdc++-6.dll"
                    Source="C:\msys64\mingw64\bin\libstdc++-6.dll"
                    DiskId="1"/>
              <File Id="WinpthreadDll"
                    Name="libwinpthread-1.dll"
                    Source="C:\msys64\mingw64\bin\libwinpthread-1.dll"
                    DiskId="1"/>
              <File Id="Bzip2Dll"
                    Name="libbz2-1.dll"
                    Source="C:\msys64\mingw64\bin\libbz2-1.dll"
                    DiskId="1"/>
              <File Id="GccsehDll"
                    Name="libgcc_s_seh-1.dll"
                    Source="C:\msys64\mingw64\bin\libgcc_s_seh-1.dll"
                    DiskId="1"/>
              <File Id="ExpatDll"
                    Name="libexpat-1.dll"
                    Source="C:\msys64\mingw64\bin\libexpat-1.dll"
                    DiskId="1"/>
              <File Id="ZlibDll"
                    Name="zlib1.dll"
                    Source="C:\msys64\mingw64\bin\zlib1.dll"
                    DiskId="1"/>
              <File Id="PngDll"
                    Name="libpng16-16.dll"
                    Source="C:\msys64\mingw64\bin\libpng16-16.dll"
                    DiskId="1"/>
              <File Id="IconvDll"
                    Name="libiconv-2.dll"
                    Source="C:\msys64\mingw64\bin\libiconv-2.dll"
                    DiskId="1"/>
              <File Id="GlibDll"
                    Name="libglib-2.0-0.dll"
                    Source="C:\msys64\mingw64\bin\libglib-2.0-0.dll"
                    DiskId="1"/>
              <File Id="GraphiteDll"
                    Name="libgraphite2.dll"
                    Source="C:\msys64\mingw64\bin\libgraphite2.dll"
                    DiskId="1"/>
              <File Id="IntlDll"
                    Name="libintl-8.dll"
                    Source="C:\msys64\mingw64\bin\libintl-8.dll"
                    DiskId="1"/>
              <File Id="PcreDll"
                    Name="libpcre-1.dll"
                    Source="C:\msys64\mingw64\bin\libpcre-1.dll"
                    DiskId="1"/>
              <File Id="Eay32Dll"
                    Name="libeay32.dll"
                    Source="C:\msys64\mingw64\bin\libeay32.dll"
                    DiskId="1"/>
              <File Id="Ssleay32Dll"
                    Name="ssleay32.dll"
                    Source="C:\msys64\mingw64\bin\ssleay32.dll"
                    DiskId="1"/>
              <File Id="HarfbuzzDll"
                    Name="libharfbuzz-0.dll"
                    Source="C:\msys64\mingw64\bin\libharfbuzz-0.dll"
                    DiskId="1"/>
              <File Id="FreetypeDll"
                    Name="libfreetype-6.dll"
                    Source="C:\msys64\mingw64\bin\libfreetype-6.dll"
                    DiskId="1"/>
              <File Id="FontconfigDll"
                    Name="libfontconfig-1.dll"
                    Source="C:\msys64\mingw64\bin\libfontconfig-1.dll"
                    DiskId="1"/>
              <File Id="AVUtil"
                    Name="avutil-55.dll"
                    Source="C:\msys64\mingw64\bin\avutil-55.dll"
                    DiskId="1"/>
	      <File Id="AVFORMAT"
		    Name="AVFORMAT-57.dll"
		    Source="c:\msys64\mingw64\bin\AVFORMAT-57.DLL"
		    DiskId="1"/>
	      <File Id="AVUTIL"
		    Name="AVUTIL-55.dll"
		    Source="c:\msys64\mingw64\bin\AVUTIL-55.DLL"
		    DiskId="1"/>
	      <File Id="LIBBLURAY"
		    Name="LIBBLURAY-1.dll"
		    Source="c:\msys64\mingw64\bin\LIBBLURAY-1.DLL"
		    DiskId="1"/>
	      <File Id="LIBCELT0"
		    Name="LIBCELT0-2.dll"
		    Source="c:\msys64\mingw64\bin\LIBCELT0-2.DLL"
		    DiskId="1"/>
	      <File Id="LIBDCADEC"
		    Name="LIBDCADEC-0.dll"
		    Source="c:\msys64\mingw64\bin\LIBDCADEC-0.DLL"
		    DiskId="1"/>
	      <File Id="LIBFFI"
		    Name="LIBFFI-6.dll"
		    Source="c:\msys64\mingw64\bin\LIBFFI-6.DLL"
		    DiskId="1"/>
	      <File Id="LIBGMP"
		    Name="LIBGMP-10.dll"
		    Source="c:\msys64\mingw64\bin\LIBGMP-10.DLL"
		    DiskId="1"/>
	      <File Id="LIBGNUTLS"
		    Name="LIBGNUTLS-30.dll"
		    Source="c:\msys64\mingw64\bin\LIBGNUTLS-30.DLL"
		    DiskId="1"/>
	      <File Id="LIBGSM"
		    Name="LIBGSM.dll"
		    Source="c:\msys64\mingw64\bin\LIBGSM.DLL"
		    DiskId="1"/>
	      <File Id="LIBHOGWEED"
		    Name="LIBHOGWEED-4-1.dll"
		    Source="c:\msys64\mingw64\bin\LIBHOGWEED-4-1.DLL"
		    DiskId="1"/>
	      <File Id="LIBIDN"
		    Name="LIBIDN-11.dll"
		    Source="c:\msys64\mingw64\bin\LIBIDN-11.DLL"
		    DiskId="1"/>
	      <File Id="LIBLZMA"
		    Name="LIBLZMA-5.dll"
		    Source="c:\msys64\mingw64\bin\LIBLZMA-5.DLL"
		    DiskId="1"/>
	      <File Id="LIBMODPLUG"
		    Name="LIBMODPLUG-1.dll"
		    Source="c:\msys64\mingw64\bin\LIBMODPLUG-1.DLL"
		    DiskId="1"/>
	      <File Id="LIBMP3LAME"
		    Name="LIBMP3LAME-0.dll"
		    Source="c:\msys64\mingw64\bin\LIBMP3LAME-0.DLL"
		    DiskId="1"/>
	      <File Id="LIBNETTLE"
		    Name="LIBNETTLE-6-1.dll"
		    Source="c:\msys64\mingw64\bin\LIBNETTLE-6-1.DLL"
		    DiskId="1"/>
	      <File Id="LIBOGG"
		    Name="LIBOGG-0.dll"
		    Source="c:\msys64\mingw64\bin\LIBOGG-0.DLL"
		    DiskId="1"/>
	      <File Id="LIBOPENCOREAMRNB"
		    Name="LIBOPENCORE-AMRNB-0.dll"
		    Source="c:\msys64\mingw64\bin\LIBOPENCORE-AMRNB-0.DLL"
		    DiskId="1"/>
	      <File Id="LIBOPENCOREAMRWB"
		    Name="LIBOPENCORE-AMRWB-0.dll"
		    Source="c:\msys64\mingw64\bin\LIBOPENCORE-AMRWB-0.DLL"
		    DiskId="1"/>
	      <File Id="LIBOPENJP"
		    Name="LIBOPENJP2-7.dll"
		    Source="c:\msys64\mingw64\bin\LIBOPENJP2-7.DLL"
		    DiskId="1"/>
	      <File Id="LIBOPUS"
		    Name="LIBOPUS-0.dll"
		    Source="c:\msys64\mingw64\bin\LIBOPUS-0.DLL"
		    DiskId="1"/>
	      <File Id="LIBORC"
		    Name="LIBORC-0.4-0.dll"
		    Source="c:\msys64\mingw64\bin\LIBORC-0.4-0.DLL"
		    DiskId="1"/>
	      <File Id="LIBP11KIT"
		    Name="LIBP11-KIT-0.dll"
		    Source="c:\msys64\mingw64\bin\LIBP11-KIT-0.DLL"
		    DiskId="1"/>
	      <File Id="LIBRTMP"
		    Name="LIBRTMP-1.dll"
		    Source="c:\msys64\mingw64\bin\LIBRTMP-1.DLL"
		    DiskId="1"/>
	      <File Id="LIBSCHROEDINGER"
		    Name="LIBSCHROEDINGER-1.0-0.dll"
		    Source="c:\msys64\mingw64\bin\LIBSCHROEDINGER-1.0-0.DLL"
		    DiskId="1"/>
	      <File Id="LIBSPEEX"
		    Name="LIBSPEEX-1.dll"
		    Source="c:\msys64\mingw64\bin\LIBSPEEX-1.DLL"
		    DiskId="1"/>
	      <File Id="LIBTASN"
		    Name="LIBTASN1-6.dll"
		    Source="c:\msys64\mingw64\bin\LIBTASN1-6.DLL"
		    DiskId="1"/>
	      <File Id="LIBTHEORADEC"
		    Name="LIBTHEORADEC-1.dll"
		    Source="c:\msys64\mingw64\bin\LIBTHEORADEC-1.DLL"
		    DiskId="1"/>
	      <File Id="LIBTHEORAENC"
		    Name="LIBTHEORAENC-1.dll"
		    Source="c:\msys64\mingw64\bin\LIBTHEORAENC-1.DLL"
		    DiskId="1"/>
	      <File Id="LIBVORBIS"
		    Name="LIBVORBIS-0.dll"
		    Source="c:\msys64\mingw64\bin\LIBVORBIS-0.DLL"
		    DiskId="1"/>
	      <File Id="LIBVORBISENC"
		    Name="LIBVORBISENC-2.dll"
		    Source="c:\msys64\mingw64\bin\LIBVORBISENC-2.DLL"
		    DiskId="1"/>
	      <File Id="LIBVPX"
		    Name="LIBVPX-1.dll"
		    Source="c:\msys64\mingw64\bin\LIBVPX-1.DLL"
		    DiskId="1"/>
	      <File Id="LIBWAVPACK"
		    Name="LIBWAVPACK-1.dll"
		    Source="c:\msys64\mingw64\bin\LIBWAVPACK-1.DLL"
		    DiskId="1"/>
	      <File Id="LIBX264"
		    Name="LIBX264-148.dll"
		    Source="c:\msys64\mingw64\bin\LIBX264-148.DLL"
		    DiskId="1"/>
	      <File Id="LIBX265"
		    Name="LIBX265.dll"
		    Source="c:\msys64\mingw64\bin\LIBX265.DLL"
		    DiskId="1"/>
	      <File Id="LIBXML2"
		    Name="LIBXML2-2.dll"
		    Source="c:\msys64\mingw64\bin\LIBXML2-2.DLL"
		    DiskId="1"/>
	      <File Id="SWRESAMPLE"
		    Name="SWRESAMPLE-2.dll"
		    Source="c:\msys64\mingw64\bin\SWRESAMPLE-2.DLL"
		    DiskId="1"/>
	      <File Id="XVIDCORE"
		    Name="XVIDCORE.dll"
		    Source="c:\msys64\mingw64\bin\XVIDCORE.DLL"
		    DiskId="1"/>
            </Component>

            <Directory Id="EtcDir" Name="etc">
              <Directory Id="FontsDir" Name="fonts">
                <Component Id="FontsDir"
                           Guid="8d37ee61-9237-438d-b976-f163bd6b0578"
                           Win64="yes">
                  <File Id="ServoFontsConfig"
                        KeyPath="yes"
                        Name="fonts.conf"
                        Source="${windowize(top_path)}\support\windows\fonts.conf"
                        DiskId="1"/>
                </Component>
              </Directory>
            </Directory>

            ${include_directory(path.join(top_path, "resources"), "resources")}
            ${include_directory(browserhtml_path, "browserhtml")}
          </Directory>
        </Directory>
      </Directory>

      <Directory Id="ProgramMenuFolder" Name="Programs">
        <Directory Id="ProgramMenuDir" Name="Servo Tech Demo">
          <Component Id="ProgramMenuDir" Guid="e04737ce-16eb-4977-9b4c-ed2db8a5a77d">
            <RemoveFolder Id="ProgramMenuDir" On="uninstall"/>
            <RegistryValue Root="HKCU"
                           Key="Software\Mozilla Research\Servo Tech Demo"
                           Type="string"
                           Value=""
                           KeyPath="yes"/>
          </Component>
        </Directory>
      </Directory>
    </Directory>

    <Feature Id="Complete" Level="1">
      <ComponentRef Id="Servo"/>
      <ComponentRef Id="FontsDir"/>
      % for c in components:
      <ComponentRef Id="${c}"/>
      % endfor
      <ComponentRef Id="ProgramMenuDir"/>
    </Feature>

    <Icon Id="Servo.ico" SourceFile="${windowize(top_path)}\resources\Servo.ico"/>
  </Product>
</Wix>
<%!
import os
import os.path as path
import re
import uuid

def make_id(s):
    return "Id{}".format(s.replace("-", "_").replace("/", "_"))

def listfiles(directory):
    return [f for f in os.listdir(directory)
            if path.isfile(path.join(directory, f))]

def listdirs(directory):
    return [f for f in os.listdir(directory)
            if path.isdir(path.join(directory, f))]

def windowize(p):
    if not p.startswith("/"):
        return p
    return re.sub("^/([^/])+", "\\1:", p)

components = []
%>
<%def name="include_directory(d, n)">
<Directory Id="${make_id(path.basename(d))}" Name="${n}">
  <Component Id="${make_id(path.basename(d))}"
             Guid="${uuid.uuid4()}"
             Win64="yes">
    <CreateFolder/>
    <% components.append(make_id(path.basename(d))) %>
    % for f in listfiles(d):
    <File Id="${make_id(path.join(d, f))}"
          Name="${f}"
          Source="${windowize(path.join(d, f))}"
          DiskId="1"/>
    % endfor
  </Component>

  % for f in listdirs(d):
  ${include_directory(path.join(d, f), f)}
  % endfor
</Directory>
</%def>
