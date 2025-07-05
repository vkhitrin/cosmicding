name := 'cosmicding'
appid := 'com.vkhitrin.cosmicding'

rootdir := ''
prefix := '/usr'

base-dir := absolute_path(clean(rootdir / prefix))

bin-src := 'target' / 'release' / name
bin-dst := base-dir / 'bin' / name

desktop := appid + '.desktop'
desktop-src := 'res' / 'linux' / desktop
desktop-dst := clean(rootdir / prefix) / 'share' / 'applications' / desktop

metainfo-dst := clean(rootdir / prefix) / 'share' / 'metainfo' / 'com.vkhitrin.cosmicding.metainfo.xml'

icons-src := 'res' / 'icons' / 'hicolor'
icons-dst := clean(rootdir / prefix) / 'share' / 'icons' / 'hicolor'

macos-assets-dir := 'res' / 'macOS'
macos-release-dir := 'target' / 'release'
macos-app-name := 'Cosmicding' + '.app'
macos-app-template := macos-assets-dir / macos-app-name
macos-app-template-plist := macos-app-template / 'Contents' / 'Info.plist'
macos-app-dir := macos-release-dir / 'macos'
macos-app-binary := macos-release-dir / name
macos-app-binary-dir := macos-app-dir / macos-app-name / 'Contents' / 'MacOS'
macos-app-extras-dir := macos-app-dir / macos-app-name / 'Contents' / 'Resources'
macos-dmg-name := name + '.dmg'
macos-dmg-release := macos-release-dir / 'macos'

default: build-release

clean:
    cargo clean

clean-vendor:
    rm -rf .cargo vendor vendor.tar

clean-dist: clean clean-vendor

build-debug *args:
    cargo build {{args}}

build-release *args:
  #!/usr/bin/env sh
  if [ "$(uname)" = "Linux" ]; then
    just build-release-linux
  elif [ "$(uname)" = "Darwin" ]; then
    export HOMEBREW_PREFIX="$(brew --prefix)"
    export PKG_CONFIG_PATH="${HOMEBREW_PREFIX}/lib/pkgconfig"
    export LIBRARY_PATH="${HOMEBREW_PREFIX}/lib"
    export C_INCLUDE_PATH="${HOMEBREW_PREFIX}/include"
    if [ "$(uname -m)" = "arm64" ]; then
        just build-release-macos-aarch64
    elif [ "$(uname -m)" = "x86_64" ]; then
        just build-release-macos-x86_64
    fi
  fi

build-release-linux *args: (build-debug '--release' args)

build-release-macos-aarch64 *args:
    #!/usr/bin/env sh
    if [ ! -z $COSMICDING_UNIVERAL_BUILD ]; then
        SDKROOT="$(xcrun --sdk macosx --show-sdk-path)"
        CFLAGS="-isysroot $SDKROOT"
        rustup run stable cargo build --release --target aarch64-apple-darwin {{args}}
    else
        cargo build --release --target=aarch64-apple-darwin {{args}}
        lipo "target/aarch64-apple-darwin/release/{{name}}" -create -output "{{macos-app-binary}}"
        just bundle-macos
    fi

build-release-macos-x86_64 *args:
    #!/usr/bin/env sh
    echo $COSMICDING_UNIVERAL_BUILD
    if [ ! -z $COSMICDING_UNIVERAL_BUILD ]; then
        SDKROOT="$(xcrun --sdk macosx --show-sdk-path)"
        CFLAGS="-isysroot $SDKROOT"
        rustup run stable cargo build --release --target x86_64-apple-darwin {{args}}
    else
        cargo build --release --target x86_64-apple-darwin {{args}}
        lipo "target/x86_64-apple-darwin/release/{{name}}" -create -output "{{macos-app-binary}}"
        just bundle-macos
    fi

build-release-macos-universal *args:
    which rustup || exit 1
    rustup toolchain install stable
    rustup target add aarch64-apple-darwin
    rustup target add x86_64-apple-darwin
    env COSMICDING_UNIVERAL_BUILD=true just build-release-macos-aarch64
    env COSMICDING_UNIVERAL_BUILD=true just build-release-macos-x86_64
    lipo "target/aarch64-apple-darwin/release/{{name}}" "target/x86_64-apple-darwin/release/{{name}}" -create -output "{{macos-app-binary}}"
    just bundle-macos

bundle-macos:
    # Using native macOS' sed
    /usr/bin/sed -i '' -e "s/__VERSION__/$(cargo pkgid | cut -d "#" -f2)/g" {{macos-app-template-plist}}
    /usr/bin/sed -i '' -e "s/__BUILD__/$(git describe --always --exclude='*')/g" {{macos-app-template-plist}}
    mkdir -p "{{macos-app-binary-dir}}"
    mkdir -p "{{macos-app-extras-dir}}/icons/hicolor"
    cp -fRp "{{macos-app-template}}" "{{macos-app-dir}}"
    cp -fp "{{macos-app-binary}}" "{{macos-app-binary-dir}}"
    cp -r ./res/icons/hicolor/* "{{macos-app-extras-dir}}/icons/hicolor"
    touch -r "{{macos-app-binary}}" "{{macos-app-dir}}/{{macos-app-name}}"
    echo "Created '{{macos-app-name}}' in '{{macos-app-dir}}'"
    git stash -- {{macos-app-template-plist}}

distribute-macos-dmg:
    which create-dmg || exit 1
    create-dmg \
      --volname "Cosmicding Installer" \
      --window-pos 200 120 \
      --window-size 800 400 \
      --icon-size 100 \
      --hide-extension "Cosmicding.app" \
      --icon {{macos-app-name}} 200 160 \
      --app-drop-link 600 155 \
      {{macos-app-dir}}/{{macos-dmg-name}} \
      {{macos-app-dir}}/{{macos-app-name}}

build-release-linux-flatpak:
    which flatpak-builder || exit 1
    flatpak-builder --force-clean \
                    --sandbox \
                    --user \
                    --install \
                    --install-deps-from=flathub \
                    --ccache \
                    --mirror-screenshots-url=https://dl.flathub.org/media/ \
                    --repo=flatpak-repo builddir \
                    res/flatpak/com.vkhitrin.cosmicding.yaml

build-vendored *args: vendor-extract (build-release '--frozen --offline' args)

check *args:
    cargo clippy --all-features {{args}} -- -W clippy::pedantic

check-json: (check '--message-format=json')

run-linux *args:
    env RUST_BACKTRACE=full cargo run --release {{args}}

run-macos:
    just build-release
    env RUST_BACKTRACE=full {{macos-app-binary-dir}}/{{name}}

run *args:
  #!/usr/bin/env sh
  if [ "$(uname)" = "Linux" ]; then
    just run-linux
  elif [ "$(uname)" = "Darwin" ]; then
    just run-macos
  fi
  
install:
    #!/usr/bin/env sh
    if [ "$(uname)" = "Linux" ]; then
        install -Dm0755 {{bin-src}} {{bin-dst}}
        find {{icons-src}} -type f -path "*/apps/*.svg" -exec bash -c '
            for src; do
                rel_path="${src#{{icons-src}}/}"
                dst="{{icons-dst}}/$rel_path"
                install -Dm0644 "$src" "$dst"
            done
        ' bash {} +
        install -Dm0644 res/linux/app.desktop {{desktop-dst}}
        install -Dm0644 res/flatpak/com.vkhitrin.cosmicding.metainfo.xml {{metainfo-dst}}
    elif [ "$(uname)" = "Darwin" ]; then
        cp -r {{macos-app-dir}}/{{name}}.app /Applications/
    fi

uninstall:
    #!/usr/bin/env sh
    if [ "$(uname)" = "Linux" ]; then
        rm {{bin-dst}} {{desktop-dst}}
        find {{icons-src}} -type f -path "*/apps/*.svg" -exec bash -c '
            for src; do
                rel_path="${src#{{icons-src}}/}"
                dst="{{icons-dst}}/$rel_path"
                if [ -f "$dst" ]; then
                    rm "$dst"
                fi
            done
        ' bash {} +
    elif [ "$(uname)" = "Darwin" ]; then
        rm -rf /Applications/{{name}}.app
    fi

vendor:
    #!/usr/bin/env bash
    mkdir -p .cargo
    cargo vendor --sync Cargo.toml | head -n -1 > .cargo/config.toml
    echo 'directory = "vendor"' >> .cargo/config.toml
    echo >> .cargo/config.toml
    echo '[env]' >> .cargo/config.toml
    if [ -n "${SOURCE_DATE_EPOCH}" ]
    then
        source_date="$(date -d "@${SOURCE_DATE_EPOCH}" "+%Y-%m-%d")"
        echo "VERGEN_GIT_COMMIT_DATE = \"${source_date}\"" >> .cargo/config.toml
    fi
    if [ -n "${SOURCE_GIT_HASH}" ]
    then
        echo "VERGEN_GIT_SHA = \"${SOURCE_GIT_HASH}\"" >> .cargo/config.toml
    fi
    tar pcf vendor.tar .cargo vendor
    rm -rf .cargo vendor

vendor-extract:
    rm -rf vendor
    tar pxf vendor.tar
