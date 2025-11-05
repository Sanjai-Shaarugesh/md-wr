# --- GTK + Blueprint + Rust Justfile ---

default: build

build-ui:
    blueprint-compiler compile src/data/ui/window.blp > src/data/ui/window.ui
    blueprint-compiler compile src/data/ui/text-editor.blp > src/data/ui/text-editor.ui

copy: build-ui
    mkdir -p build/schemas
    cp src/data/org.md-wr.com.gschema.xml build/schemas/
    glib-compile-schemas build/schemas

build-resources: copy
    glib-compile-resources --target=resources.gresource --sourcedir=src/data/ui src/data/ui/resources.gresource.xml

build: build-resources
    cargo build --release

run: build
    env GSETTINGS_SCHEMA_DIR="$(pwd)/build/schemas:$(pkg-config --variable=schemasdir gio-2.0 || echo /usr/share/glib-2.0/schemas)" \
    LD_LIBRARY_PATH="/usr/lib" \
    cargo run

test:
    cargo fmt -- --check
    cargo clippy -- -D warnings
    cargo test
