# Compile all blueprint files to GtkBuilder XML
build-ui:
    blueprint-compiler compile src/data/ui/window.blp > src/data/ui/window.ui
    blueprint-compiler compile src/data/ui/text-editor.blp > src/data/ui/text-editor.ui
    
    
  

# Compile resources after building UI
build-resources: build-ui
    glib-compile-resources --target=resources.gresource --sourcedir=src/data/ui src/data/ui/resources.gresource.xml
    # Compile GSettings schema
    glib-compile-schemas src/data --targetdir=src/data/ui

    # Compile resources
    glib-compile-resources --target=resources.gresource --sourcedir=src/data/ui src/data/ui/resources.gresource.xml

# Run your Rust app (compile UI and resources first)
run: build-resources
    cargo build
    cargo run