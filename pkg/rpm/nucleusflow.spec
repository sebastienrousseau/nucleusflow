Name:           nucleusflow
Version:        0.0.10
Release:        1%{?dist}
Summary:        High-performance, memory-safe static site generator written in Rust
License:        Apache-2.0 OR MIT
URL:            https://nucleusflow.com
Source0:        https://github.com/sebastienrousseau/%{name}/archive/refs/tags/v%{version}.tar.gz

BuildRequires:  cargo
BuildRequires:  rust >= 1.89.0

%description
NucleusFlow is a high-performance, memory-safe static site generator and content
processing pipeline written in Rust with strict zero-unsafe code guarantees.

%prep
%autosetup

%build
cargo build --release --bin %{name}

%install
make -f GNUmakefile PREFIX=%{_prefix} DESTDIR=%{buildroot} install

%check
cargo test

%files
%license LICENSE-APACHE LICENSE-MIT
%doc README.md CHANGELOG.md
%{_bindir}/%{name}
%{_mandir}/man1/%{name}.1*
%{_datadir}/bash-completion/completions/%{name}
%{_datadir}/zsh/site-functions/_%{name}
%{_datadir}/fish/vendor_completions.d/%{name}.fish
