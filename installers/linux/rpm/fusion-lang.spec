Name:           fusion-lang
Version:        2.0.0
Release:        1
Summary:        Fusion v2.0 Vortex Programming Language

License:        MIT
URL:            https://github.com/QuantumSecureTechnologiesInc/Fusion-Programming-Language
Source0:        fusion-lang-2.0.0.tar.gz

BuildRequires:  gcc, cmake, openssl-devel
Requires:       glibc, libstdc++

%description
A modern, polyglot systems programming language with post-quantum cryptography.

%prep
%setup -q

%build
make %{?_smp_mflags}

%install
mkdir -p %{buildroot}/opt/fusion/bin
install -m 755 fuc %{buildroot}/opt/fusion/bin/
install -m 755 fusion %{buildroot}/opt/fusion/bin/
mkdir -p %{buildroot}/opt/fusion/stdlib
cp -r stdlib/* %{buildroot}/opt/fusion/stdlib/
