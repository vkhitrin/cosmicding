> [!CAUTION]
> This branch is being worked. The application does not behave properly!

<p align="center">
  <img alt="cosmicding logo" src="./res/icons/hicolor/256x256/apps/com.vkhitrin.cosmicding.png" alt="Logo" height="192px" width="192px">
</p>

<p align="center">
    <img alt="cosmicding accounts page" src="./res/screenshots/accounts.png" width="192">
    <img alt="cosmicding bookmarks page" src="./res/screenshots/bookmarks.png" width="192">
</p>

# cosmicding

cosmicding is a [linkding](https://github.com/sissbruecker/linkding) companion app for COSMIC™ Desktop Environment.  
It provides an alternative frontend to linkding based on [libcosmic](https://github.com/pop-os/libcosmic).

While cosmicding was desgined for COSMIC™ Desktop Environment, it should be able to run cross-platform.

Features:

- Support multiple linkding instances (or multiple users on the same instance).
- Cache/aggregate bookmarks locally.
- Add/Edit/Remove bookmarks.
- Search bookmarks based on title, URL, tags, desscription, and notes.

cosmicding was tested against linkding releases `1.31.0`, and `1.36.0`.

## Dependencies

- `cargo`
- `just`
- `libxkbcommon-dev`
- `libcosmic`
- `libsqlite3-dev`

## Installation

### Local

```shell
# Clone the repository
git clone https://github.com/vkhitrin/cosmicding

# Change directory to the project folder
cd cosmicding

# Build Release version
just build-release

# Install
sudo just install
```

## Roadmap

### Future

Potential improvements:

- [UI] Detailed `About` page.
- [Performance] Check performance with multiple remote + local instances.
- [Performance] Check performance with high amount of bookmarks spread across multiple instances.
- [Application] Refactor codebase to be more organized.
- [Application] Allow user-provided TLS certificate.
- [UI] Visual indicator for last sync status.
- [UI] Indicators for `archived`, `unread`, `shared` bookmarks.
- [UI] Display account information in accounts page.
- [Distribution] Flatpack release.
- [Distribution] compiled binary in GitHub release.
- [UI] Sort bookmarks.

Things to consider:

- [UI] Add context menus (right click) for accounts/bookmarks.
- [Application] Periodic auto refresh (sync with remote).
- [UI] Pagination (if possible).
- [Application] Consider leveraging linkding's `/check` endpoint when adding bookmarks.
- [Application] Do not block on when executing local database queries.

## Thanks

- [cosmic-utils](https://github.com/cosmic-utils) organization for their code examples.
- [@sissbruecker](https://github.com/sissbruecker) for creating linkding.
- [system76](https://system76.com) for creating COSMIC, and making it fun to develop for.
