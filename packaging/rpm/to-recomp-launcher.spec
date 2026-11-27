# Built from a tagged checkout; see packaging/rpm/README.md and docs/releasing.md.
# The build needs network access (npm and crates.io): enable "internet access" on the COPR
# project, or pass --enable-network to mock.

%global debug_package %{nil}

# release-please keeps the line between the markers equal to the app version (package.json,
# tauri.conf.json, Cargo.toml). A pre-release 1.2.3-rc.1 is packaged as Version 1.2.3, Release
# 0.rc.1 (it sorts before 1.2.3-1).
# x-release-please-start-version
%global upstream_version 0.2.0
# x-release-please-end
%global rpm_version %(echo %{upstream_version} | cut -d- -f1)
%global rpm_release %(echo %{upstream_version} | awk -F- 'NF > 1 { print "0." $2; next } { print 1 }')

Name:           to-recomp-launcher
Version:        %{rpm_version}
Release:        %{rpm_release}%{?dist}
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
* Thu Nov 26 2026 To Recomp maintainers <noreply@users.noreply.github.com> - %{version}-%{release}
- Built from the release tag. The changes of every version are in CHANGELOG.md and on the GitHub
  release page; this entry follows the version so no per-release edit of the spec is needed.

* Thu Nov 26 2026 To Recomp maintainers <noreply@users.noreply.github.com> - 0.1.0-1
- First stable release (0.1.0): install, prepare, configure and launch the Tales of recompilations from
  one launcher. Games update from their releases, saves can be backed up, restored and cleaned, and the
  launcher updates itself (the AppImage and the Windows installer, from signed releases).

* Thu Nov 26 2026 To Recomp maintainers <noreply@users.noreply.github.com> - 0.1.0-0.5.rc.5
- Fifth pre-release (0.1.0-rc.5): the first build signed for the updater (the AppImage and the Windows
  installer can update themselves once a stable release is published). Nothing else changes since rc.4.

* Thu Nov 26 2026 To Recomp maintainers <noreply@users.noreply.github.com> - 0.1.0-0.4.rc.4
- Fourth pre-release (0.1.0-rc.4): backing up, restoring and cleaning a game's saves in its Saves tab,
  the game's settings in the Play button's menu, tabs without settings disabled, the chosen files kept
  when the build is deleted (and only the missing ones asked for), a console's state and maturity in the
  platform selector, Lucide icons, bundled fonts (Inter, JetBrains Mono) and a game-coloured settings page.

* Wed Nov 25 2026 To Recomp maintainers <noreply@users.noreply.github.com> - 0.1.0-0.3.rc.3
- Third pre-release (0.1.0-rc.3): update detection and an Update button for installed games, quick
  play from the sidebar, a Files section in the game settings (one Change button per file, no automatic
  rebuild), four launcher settings cards, no light theme, far fewer GitHub requests, a Cancel and a
  log panel that work, the tray's left click.

* Tue Nov 24 2026 To Recomp maintainers <noreply@users.noreply.github.com> - 0.1.0-0.2.rc.2
- Second pre-release (0.1.0-rc.2): game settings page with a Files tab, hot-swapped language,
  quick play from the sidebar, cached GitHub requests, release_year per console.

* Mon Nov 23 2026 To Recomp maintainers <noreply@users.noreply.github.com> - 0.1.0-0.1.rc.1
- First pre-release (0.1.0-rc.1), to try the release flow.

* Sun Nov 22 2026 To Recomp maintainers <noreply@users.noreply.github.com> - 0.1.0-1
- Initial package
