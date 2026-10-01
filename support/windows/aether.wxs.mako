<Wix xmlns="http://wixtoolset.org/schemas/v4/wxs">
  <Package Name="Aether"
           Manufacturer="The Aether Project Developers"
           UpgradeCode="7c3a9e2f-4b81-4d6a-9f13-2e8c5a1b0d47"
           Language="1033"
           Codepage="1252"
           Version="0.7.0"
           InstallerVersion="200">
    <SummaryInformation Keywords="Installer"
                        Description="Aether Installer"
                        Manufacturer="The Aether Project Developers"/>
    <MajorUpgrade AllowDowngrades="yes"/>
    <Media Id="1"
           Cabinet="Aether.cab"
           EmbedCab="yes"/>
    <StandardDirectory Id="ProgramFiles64Folder">
      <Directory Id="Aether" Name="Aether">
        <Directory Id="INSTALLDIR" Name="Aether">
            <Component Id="Aether"
                       Guid="a14f6c82-9e30-4c5b-b7d1-6f2a8e4c91b5"
                       Bitness="always64">
              <File Id="AetherEXE"
                    Name="${binary_name}"
                    DiskId="1"
                    Source="${windowize(exe_path)}\${binary_name}"
                    KeyPath="yes">
              </File>
	            ${include_dependencies()}
            </Component>

            ${include_directory(resources_path, "resources")}
          </Directory>
        </Directory>
      </StandardDirectory>

      <StandardDirectory Id="ProgramMenuFolder">
        <Directory Id="ProgramMenuDir" Name="Aether">
          <Component Id="ProgramMenuDir" Guid="d8b2e1a4-5c70-4f19-8a3e-1b6d9c4f2e80">
            <RemoveFolder Id="ProgramMenuDir" On="both"/>
            <RegistryValue Root="HKCU"
                           Key="Software\Aether\Aether"
                           Type="string"
                           Value=""
                           KeyPath="yes"/>
            <Shortcut Id="StartMenuAether"
              Directory="ProgramMenuDir"
              Name="Aether"
              Target="[INSTALLDIR]${binary_name}"
              WorkingDirectory="INSTALLDIR"
              Icon="${binary_name}"/>
          </Component>
        </Directory>
      </StandardDirectory>

      <Feature Id="Complete" Level="1">
        <ComponentRef Id="Aether"/>
         % for c in components:
         <ComponentRef Id="${c}"/>
         % endfor
        <ComponentRef Id="ProgramMenuDir"/>
      </Feature>

      <Icon Id="${binary_name}" SourceFile="${windowize(exe_path)}\${binary_name}"/>
    </Package>
</Wix>
<%!
import os
import os.path as path
import re
import uuid

def make_id(s):
    s = s.replace("-", "_").replace("/", "_").replace("\\", "_")
    return "Id{}".format(s)

def listfiles(directory):
    return [f for f in os.listdir(directory)
            if path.isfile(path.join(directory, f))]

def listdirs(directory):
    return [f for f in os.listdir(directory)
            if path.isdir(path.join(directory, f))]

def listdeps(temp_dir, exe_name):
    return [path.join(temp_dir, f) for f in os.listdir(temp_dir) if os.path.isfile(path.join(temp_dir, f)) and f != exe_name]

def windowize(p):
    if not p.startswith("/"):
        return p
    return re.sub("^/([^/])+", "\\1:", p)

components = []
%>

<%def name="include_dependencies()">
% for f in listdeps(dir_to_temp, binary_name):
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
             Bitness="always64">
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
