name := 'cosmicding'
appid := 'com.vkhitrin.cosmicding'
migrations_folder := clean(rootdir / prefix) / 'share' / appid / 'migrations'

rootdir := ''
prefix := '/usr'

base-dir := absolute_path(clean(rootdir / prefix))

bin-src := 'target' / 'release' / name
bin-dst := base-dir / 'bin' / name

desktop := appid + '.desktop'
desktop-src := 'res' / 'linux' / desktop
desktop-dst := clean(rootdir / prefix) / 'share' / 'applications' / desktop

icons-src := 'res' / 'linux' / 'icons' / 'hicolor'
icons-dst := clean(rootdir / prefix) / 'share' / 'icons' / 'hicolor'

macos-assets-dir := 'res' / 'macOS'
macos-release-dir := 'target' / 'release'
macos-app-name := name + '.app'
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
    just build-release-macos
  fi
build-release-linux *args: (build-debug '--release' args)

build-release-macos *args:
    cargo build --release --target=aarch64-apple-darwin {{args}}

    # Using native macOS' sed
    /usr/bin/sed -i '' -e "s/__VERSION__/$(cargo pkgid | cut -d "#" -f2)/g" {{macos-app-template-plist}}
    /usr/bin/sed -i '' -e "s/__BUILD__/$(git describe --always --exclude='*')/g" {{macos-app-template-plist}}

    lipo "target/aarch64-apple-darwin/release/{{name}}" -create -output "{{macos-app-binary}}"

    mkdir -p "{{macos-app-binary-dir}}"
    mkdir -p "{{macos-app-extras-dir}}/icons/"
    cp -fRp "{{macos-app-template}}" "{{macos-app-dir}}"
    cp -fp "{{macos-app-binary}}" "{{macos-app-binary-dir}}"
    cp ./res/icons/* "{{macos-app-extras-dir}}/icons/"
    touch -r "{{macos-app-binary}}" "{{macos-app-dir}}/{{macos-app-name}}"
    echo "Created '{{macos-app-name}}' in '{{macos-app-dir}}'"
    git stash -- {{macos-app-template-plist}}

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
        just install-migrations
        install -Dm0755 {{bin-src}} {{bin-dst}}
        install -Dm0644 res/linux/app.desktop {{desktop-dst}}
        for size in `ls {{icons-src}}`; do \
            install -Dm0644 "{{icons-src}}/$size/apps/{{appid}}.png" "{{icons-dst}}/$size/apps/{{appid}}.png"; \
        done
    elif [ "$(uname)" = "Darwin" ]; then
        cp -r {{macos-app-dir}}/{{name}}.app /Applications/
    fi

install-migrations:
  #!/usr/bin/env sh
  set -ex
  for file in ./migrations/*; do
    install -Dm0644 $file "{{migrations_folder}}/$(basename "$file")"
  done

uninstall:
    #!/usr/bin/env sh
    if [ "$(uname)" = "Linux" ]; then
        rm {{bin-dst}} {{desktop-dst}}
        for size in `ls {{icons-src}}`; do \
            rm "{{icons-dst}}/$size/apps/{{appid}}.png"; \
        done
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
