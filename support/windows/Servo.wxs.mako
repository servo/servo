<?xml version="1.0" encoding="utf-8"?>
<Wix xmlns="http://schemas.microsoft.com/wix/2006/wi">
  <Product Id="*"
           Name="Servo Tech Demo"
           Manufacturer="Mozilla Research"
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
    <MajorUpgrade AllowDowngrades="yes"/>
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
                          Advertise="yes"/>
              </File>
              ${include_dependencies()}
            </Component>

            ${include_directory(resources_path, "resources")}
            ${include_directory(path.join(dir_to_temp, "browserhtml"), "browserhtml")}
          </Directory>
        </Directory>
      </Directory>

      <Directory Id="ProgramMenuFolder" Name="Programs">
        <Directory Id="ProgramMenuDir" Name="Servo Tech Demo">
          <Component Id="ProgramMenuDir" Guid="e04737ce-16eb-4977-9b4c-ed2db8a5a77d">
            <RemoveFolder Id="ProgramMenuDir" On="both"/>
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

    <Icon Id="Servo.ico" SourceFile="${windowize(resources_path)}\Servo.ico"/>
  </Product>
</Wix>
<%!
import os
import os.path as path
import re
import uuid
from servo.command_base import host_triple

def make_id(s):
    s = s.replace("-", "_").replace("/", "_").replace("\\", "_")
    return "Id{}".format(s)

def listfiles(directory):
    return [f for f in os.listdir(directory)
            if path.isfile(path.join(directory, f))]

def listdirs(directory):
    return [f for f in os.listdir(directory)
            if path.isdir(path.join(directory, f))]

def listdeps(temp_dir):
    return [path.join(temp_dir, f) for f in os.listdir(temp_dir) if os.path.isfile(path.join(temp_dir, f)) and f != "servo.exe"]

def windowize(p):
    if not p.startswith("/"):
        return p
    return re.sub("^/([^/])+", "\\1:", p)

components = []
%>

<%def name="include_dependencies()">
% for f in listdeps(dir_to_temp):
              <File Id="${make_id(path.basename(f)).replace(".","").replace("+","x")}"
                    Name="${path.basename(f)}"
                    Source="${f}"
                    DiskId="1"/>
% endfor
</%def>

<%def name="include_directory(d, n)">
<Directory Id="${make_id(path.basename(d))}" Name="${n}">
  <Component Id="${make_id(path.basename(d))}"
             Guid="${uuid.uuid4()}"
             Win64="yes">
    <CreateFolder/>
    <% components.append(make_id(path.basename(d))) %>
    % for f in listfiles(d):
    <File Id="${make_id(path.join(d, f).replace(dir_to_temp, ""))}"
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
