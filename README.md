<p align="center">
  <img alt="cosmicding logo" src="./res/icons/hicolor/256x256/apps/com.vkhitrin.cosmicding.png" alt="Logo" height="192px" width="192px">
</p>

> [!CAUTION]
> This application is still under development, and is not deemed stable for general use.

<p align="center">
    <img alt="cosmicding accounts page" src="./res/screenshots/accounts.png" width="192">
    <img alt="cosmicding bookmarks page" src="./res/screenshots/bookmarks.png" width="192">
</p>

# cosmicding

cosmicding is a [linkding](https://github.com/sissbruecker/linkding) companion app for COSMIC™ Desktop Environment.  
It provides an alternative frontend to linkding based on [libcosmic](https://github.com/pop-os/libcosmic).

Features:

- Support multiple linkding instances (or users on the same instance).
- Cache/aggregate bookmarks locally.
- Add/Edit/Remove bookmarks.
- Search bookmarks based on title, URL, desscription and notes.

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

### Initial Stable Release

> [!NOTE]
> This list is not complete and subject to change.

In preparation for the first initial "stable" release, the following must be completed:

- [ ] [Application] Ensure no empty values are passed during creation.
  - [ ] Database schema check.
  - [ ] Application side validation.
- [ ] [Application] Throw exceptions when failed to execute database queries.
- [ ] [Application] Throw exceptions when failed to invoke REST API requests.
- [ ] [UI] Notifications/toasts.
- [ ] [Application] logging.

### Future

- [UI] Add context menus (right click) for accounts/bookmarks (?).
- [UI] Populate menu bar with more actions.
- [UI] Detailed `About` page.
- [Application] Periodic auto refresh (sync with remote) (?).
- [Performance] Check performance with multiple remote + local instances.
- [Performance] Check performance with high amount of bookmarks spread across multiple instances.
- [Application] Refactor codebase to be more organized.
- [UI] Pagination (if possible).
- [Application] use user-provided TLS certificate.
- [UI] Visual indicator for last sync status.

## Thanks

- [cosmic-utils](https://github.com/cosmic-utils) organization for their code examples.
- [@sissbruecker](https://github.com/sissbruecker) for creating linkding.
- [system76](https://system76.com) for creating COSMIC, and making it fun to develop for.
