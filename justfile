name := 'cosmicding'
appid := 'com.vkhitrin.cosmicding'
migrations_folder := clean(rootdir / prefix) / 'share' / appid / 'migrations'

rootdir := ''
prefix := '/usr'

base-dir := absolute_path(clean(rootdir / prefix))

bin-src := 'target' / 'release' / name
bin-dst := base-dir / 'bin' / name

desktop := appid + '.desktop'
desktop-src := 'res' / desktop
desktop-dst := clean(rootdir / prefix) / 'share' / 'applications' / desktop

icons-src := 'res' / 'icons' / 'hicolor'
icons-dst := clean(rootdir / prefix) / 'share' / 'icons' / 'hicolor'

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

check *args:
    cargo clippy --all-features {{args}} -- -W clippy::pedantic

check-json: (check '--message-format=json')

run *args:
    env RUST_BACKTRACE=full cargo run --release {{args}}

install: install-migrations
    install -Dm0755 {{bin-src}} {{bin-dst}}
    install -Dm0644 res/app.desktop {{desktop-dst}}
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
