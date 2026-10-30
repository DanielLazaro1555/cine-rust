# Cine (Rust Edition)

Reescritura nativa en **Rust** (`gtk4-rs` + `libadwaita-rs` + `libmpv` FFI) del reproductor de video **Cine** de [Diego Povliuk](https://github.com/diegopvlk/Cine).

## Características de esta versión en Rust
- **Binario nativo compilado** sin necesidad de intérprete de Python.
- **Interfaz GTK4 + Libadwaita** idéntica al proyecto original (`cine.gresource` embebido en el ejecutable).
- **Seguridad de memoria y ciclo de vida (`Drop`):** Manejo seguro de `GtkApplication::inhibit` / `uninhibit` y cierre limpio de `libmpv` sin fallos de segmentación (`SIGSEGV`).
- **Aceleración por hardware (`hwdec`):** Soporte directo para `libmpv` con `OpenGL` (`GtkGLArea` + `GtkGraphicsOffload`).

## Estructura del proyecto (Paso a Paso)
- `src/main.rs`: Punto de entrada, registro de recursos embebidos (`cine.gresource`), configuración de `Adw::Application` y acciones globales (`app.new-window`, `app.preferences`, `app.about`, `app.quit`).
- `src/window.rs` y `src/window/`: Ventana principal (`CineWindow`) y módulos separados para interfaz, reproducción, controles, diálogos y acciones.
- `src/mpv_gl_area.rs`: Integración FFI con `libmpv` y `libEGL`/`libGL` sobre `GtkGLArea`.
- `install.sh`: Compila e instala Cine en el perfil del usuario, incluyendo el lanzador del menú.
- `resources/`: Archivos de interfaz `.blp` / `.ui`, hojas de estilo `style.css` y shaders.
- `data/`: Iconos, archivo `.desktop`, esquema `GSettings` y metadatos AppStream.

## Instalar o actualizar en el equipo

Desde la raíz del repositorio, ejecuta:

```bash
./install.sh
```

El script compila la versión optimizada, la copia a `~/.local/bin/cine` y actualiza el lanzador. Puedes repetir el mismo comando cada vez que quieras instalar los cambios recientes.

## Licencia y Créditos
Distribuido bajo la licencia **GPL-3.0-or-later**.
Diseño original y recursos por **Diego Povliuk** ([diegopvlk/Cine](https://github.com/diegopvlk/Cine)).
