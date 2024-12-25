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

# NOTE: macOS related, should be consolidated
assets-dir := 'res' / 'macOS'
release-dir := 'target' / 'release'
app-name := name + '.app'
app-template := assets-dir / app-name
app-template-plist := app-template / 'Contents' / 'Info.plist'
app-dir := release-dir / 'macos'
app-binary := release-dir / name
app-binary-dir := app-dir / app-name / 'Contents' / 'MacOS'
app-extras-dir := app-dir / app-name / 'Contents' / 'Resources'
dmg-name := name + '.dmg'
dmg-release := release-dir / 'macos'
version := '0.1.0'
build := 'test'

default: build-release

clean:
    cargo clean

clean-vendor:
    rm -rf .cargo vendor vendor.tar

clean-dist: clean clean-vendor

build-debug *args:
    cargo build {{args}}

build-release *args: (build-debug '--release' args)

build-vendored *args: vendor-extract (build-release '--frozen --offline' args)

build-macos:
    cargo build --release --target=aarch64-apple-darwin
    lipo "target/aarch64-apple-darwin/release/{{name}}" -create -output "{{app-binary}}"

    mkdir -p "{{app-binary-dir}}"
    mkdir -p "{{app-extras-dir}}"
    cp -fRp "{{app-template}}" "{{app-dir}}"
    cp -fp "{{app-binary}}" "{{app-binary-dir}}"
    touch -r "{{app-binary}}" "{{app-dir}}/{{app-name}}"
    echo "Created '{{app-name}}' in '{{app-dir}}'"

check *args:
    cargo clippy --all-features {{args}} -- -W clippy::pedantic

check-json: (check '--message-format=json')

run *args:
    env RUST_BACKTRACE=full cargo run --release {{args}}

install: install-migrations
    install -Dm0755 {{bin-src}} {{bin-dst}}
    install -Dm0644 res/linux/app.desktop {{desktop-dst}}
    for size in `ls {{icons-src}}`; do \
        install -Dm0644 "{{icons-src}}/$size/apps/{{appid}}.png" "{{icons-dst}}/$size/apps/{{appid}}.png"; \
    done

install-migrations:
  #!/usr/bin/env sh
  set -ex
  for file in ./migrations/*; do
    install -Dm0644 $file "{{migrations_folder}}/$(basename "$file")"
  done

uninstall:
    rm {{bin-dst}} {{desktop-dst}}
    for size in `ls {{icons-src}}`; do \
        rm "{{icons-dst}}/$size/apps/{{appid}}.png"; \
    done

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
