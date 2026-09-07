Name:           macsmc-fans-cc-plugin
Version:        __VERSION__
Release:        0%{?dist}
Summary:        Plugin for CoolerControl for Apple Silicon Macs

License:        GPLv3

URL:            https://github.com/sagebind/macsmc-fans-cc-plugin
Source0:        %{name}-%{version}.tar.gz

BuildRequires:  cargo
BuildRequires:  protobuf-compiler

%description
Plugin for CoolerControl for Apple Silicon Macs.

%prep
%setup

%build
make build

%install
make DESTDIR=%{buildroot} install

%files
%license LICENSE
%doc README.md
/var/lib/coolercontrol/plugins/macsmc-fans/manifest.toml
/var/lib/coolercontrol/plugins/macsmc-fans/macsmc-fans

%changelog
%autochangelog
