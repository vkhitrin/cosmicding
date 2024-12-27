<p align="center">
  <img alt="cosmicding logo" src="./res/linux/icons/hicolor/256x256/apps/com.vkhitrin.cosmicding.png" alt="Logo" height="192px" width="192px">
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
- Aggregate bookmarks locally.
- Add/Edit/Remove bookmarks.
- Search bookmarks based on title, URL, tags, desscription, and notes.

cosmicding was tested against linkding releases `1.31.0`, and `1.36.0`.

## Installation

cosmicding is not distributed at the moment, and has to be built manually.

### Local (Linux)

Dependencies:

- `cargo`
- `just`
- `libxkbcommon-dev`
- `libcosmic`
- `libsqlite3-dev`

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

### Local (macOS)

Dependencies:

- `cargo`
- `just`
- `libxkbcommon`
- `sqlite3`

```shell
# Clone the repository
git clone https://github.com/vkhitrin/cosmicding

# Change directory to the project folder
cd cosmicding

# Build Release version
just build-macos

# Install globally (for all users)
cp -r target/release/macos/cosmicding.app /Applications
```

## Roadmap

cosmicding is currently under heavy development, and is not distributed outside of source code.

The initial release is expected to support macOS and Linux platforms.

### Future

Things to consider:

- [Application] "Move" (Delete from origin, and re-create on destination) bookmarks between accounts.

## Thanks

- [cosmic-utils](https://github.com/cosmic-utils) organization for their code examples.
- [@sissbruecker](https://github.com/sissbruecker) for creating linkding.
- [system76](https://system76.com) for creating COSMIC, and making it fun to develop for.
