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
            </Component>

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
