# Compile all Blueprint files to GtkBuilder XML
build-ui:
	blueprint-compiler compile src/data/ui/window.blp > src/data/ui/window.ui
	blueprint-compiler compile src/data/ui/text-editor.blp > src/data/ui/text-editor.ui

# Compile local GSettings schema without sudo
copy: build-ui
	mkdir -p build/schemas
	cp src/data/org.md-wr.com.gschema.xml build/schemas/
	glib-compile-schemas build/schemas

# Compile resources after building UI and schemas
build-resources: copy
	glib-compile-resources --target=resources.gresource --sourcedir=src/data/ui src/data/ui/resources.gresource.xml

# Run  app with fallback to system schemas
run: build-resources
	cargo build
	env GSETTINGS_SCHEMA_DIR="$(pwd)/build/schemas:$(pkg-config --variable=schemasdir gio-2.0 || echo /usr/share/glib-2.0/schemas)" \
	LD_LIBRARY_PATH="/usr/lib" \
	cargo run
