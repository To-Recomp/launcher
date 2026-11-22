# Built from a tagged checkout; see packaging/rpm/README.md and docs/releasing.md.
# The build needs network access (npm and crates.io): enable "internet access" on the COPR
# project, or pass --enable-network to mock.

%global debug_package %{nil}

Name:           to-recomp-launcher
Version:        0.1.0
Release:        1%{?dist}
Summary:        Install, prepare, configure and launch the Tales of recompilations
License:        GPL-3.0-only
URL:            https://github.com/To-Recomp/launcher
Source0:        launcher-%{version}.tar.gz

BuildRequires:  gcc
BuildRequires:  cargo
BuildRequires:  rust
BuildRequires:  nodejs
BuildRequires:  npm
BuildRequires:  desktop-file-utils
BuildRequires:  pkgconfig(webkit2gtk-4.1)
BuildRequires:  pkgconfig(gtk+-3.0)
BuildRequires:  pkgconfig(librsvg-2.0)
BuildRequires:  pkgconfig(libsoup-3.0)
BuildRequires:  pkgconfig(javascriptcoregtk-4.1)

Requires:       webkit2gtk4.1
Requires:       gtk3
Requires:       libayatana-appindicator-gtk3
Requires:       hicolor-icon-theme

%description
To Recomp Launcher installs, prepares, configures and launches the static
recompilations of the main Tales of games. You bring your own copy of the game;
the launcher contains no game files.

Not affiliated with or endorsed by Bandai Namco Entertainment or any game
publisher or developer.

%prep
%autosetup -n launcher-%{version}

%build
npm ci
npm run build
# Same as `tauri build --no-bundle`: embed the built frontend instead of the dev server.
cargo build --release --locked --manifest-path src-tauri/Cargo.toml --features tauri/custom-protocol

%install
install -Dm0755 src-tauri/target/release/to-recomp-launcher %{buildroot}%{_bindir}/to-recomp-launcher
install -Dm0644 packaging/linux/org.torecomp.launcher.desktop \
    %{buildroot}%{_datadir}/applications/org.torecomp.launcher.desktop
install -Dm0644 src-tauri/icons/icon.png \
    %{buildroot}%{_datadir}/icons/hicolor/512x512/apps/org.torecomp.launcher.png
install -Dm0644 src-tauri/icons/128x128.png \
    %{buildroot}%{_datadir}/icons/hicolor/128x128/apps/org.torecomp.launcher.png

%check
desktop-file-validate %{buildroot}%{_datadir}/applications/org.torecomp.launcher.desktop

%files
%license LICENSE
%doc README.md
%{_bindir}/to-recomp-launcher
%{_datadir}/applications/org.torecomp.launcher.desktop
%{_datadir}/icons/hicolor/*/apps/org.torecomp.launcher.png

%changelog
* Sun Nov 22 2026 To Recomp maintainers <noreply@users.noreply.github.com> - 0.1.0-1
- Initial package
