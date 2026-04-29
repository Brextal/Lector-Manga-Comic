# Lector Manga/Comic

Un lector de PDF, CBZ, CBR para cómics/mangas, construido con Rust y el framework eframe/egui, diseñado para lectura de manga y comics en distros basadas en Arch.

![Lector Manga/Comic](icons/lector.png)

## Características

- Navegación de archivos integrada
- Control de zoom y páginas
- Guardado automático de progreso por archivo
- Interfaz fluida y responsiva
- Navegación completa con teclado
- Soporte para archivos con caracteres especiales (ej: `#`)
- Zoom optimizado (CBZ/CBR instantáneo con GPU)
- Desplazamiento suave con flechas (igual que scroll del mouse)

## Instalación

### Requisitos

- Linux (probado en Arch Linux con Hyprland)
- Rust (si compilás desde código fuente)

### Método 1: Binario precompilado
Descarga el binario de Releases y ejecutá:
```bash
sudo cp lector-pdf /usr/local/bin/lector
```
dar permisos:
```bash
sudo chmod +x /usr/local/bin/lector
```
se lanza desde terminal: 
```bash
lector
```

### Método 2: Compilación desde código fuente
Clonar el repositorio
```bash
git clone https://github.com/Brextal/Lector-Manga-Comic.git
```
Ir a directorio Lector-Manga-Comic
```bash
cd Lector-Manga-Comic
```

### Compilar
```bash
cargo build --release
```
# Instalar
```bash
sudo cp target/release/lector-pdf /usr/local/bin/lector
```
dar permisos
```bash
sudo chmod +x /usr/local/bin/lector
```

## Configuración del Launcher (Linux)

El archivo `.desktop` ya está incluido para aparecer en tu launcher de aplicaciones.

### Icono

El icono se encuentra en `icons/lector.png`. Para que aparezca correctamente:

```bash
mkdir -p ~/.local/share/icons/hicolor/128x128/apps
cp icons/lector.png ~/.local/share/icons/hicolor/128x128/apps/lector.png
```

## Uso

1. Ejecutá `lector` o buscá "Lector Manga/Comic" en tu launcher
2. Navegá por tus carpetas usando el teclado o mouse:
   - **Flechas Arriba/Abajo** - Navegar entre archivos/directorios
   - **Enter** - Entrar a directorio / Abrir archivo
   - **Backspace** - Subir un directorio (límite: `/home/brextal/`)
   - **Botón `^`** - Subir un directorio

3. Una vez abierto un archivo:
   - **Flechas Izquierda/Derecha** - Página anterior/siguiente
   - **Campo de texto** - Escribí el número de página + **Enter** o botón "Ir"
   - **Flechas Arriba/Abajo** - Desplazamiento vertical (scroll continuo, igual que mouse wheel)
   - **`+` / `-`** - Zoom in/out
   - **`Q` o `Escape`** - Volver al navegador de archivos

## Formatos Soportados

- **PDF** - Usa poppler-rs para renderizado de alta calidad
- **CBZ (ZIP)** - Archivos ZIP con imágenes
- **CBR (RAR)** - Archivos RAR con imágenes (extrae a directorio temporal único)

## Estructura del Proyecto

```
lector-pdf/
├── src/
│   ├── main.rs          # Punto de entrada
│   ├── lib.rs           # Módulos del proyecto
│   ├── app_state.rs    # Gestión de estado/guardado
│   ├── file_browser.rs # Navegador de archivos (teclado + mouse)
│   ├── viewer.rs       # Trait Viewer y navegación común
│   ├── pdf_viewer.rs    # Visor de PDFs (codificación URI para `#`)
│   ├── comic_viewer.rs  # Visor de CBZ (zoom optimizado GPU)
│   └── cbr_viewer.rs   # Visor de CBR (hash-based temp dirs)
├── icons/               # Iconos de la aplicación
├── Cargo.toml          # Dependencias
└── README.md           # Este archivo
```

## Rendimiento

- **CBZ/CBR**: Zoom instantáneo (texturas a resolución completa, escalado por GPU)
- **PDF**: Renderizado por nivel de zoom (calidad óptima)
- **Caché inteligente**: Mantiene 2 páginas vecinas en memoria
- **Guardado automático**: Progreso se guarda cada 500ms

## Licencia

MIT

## Contribuciones

¡Las contribuciones son bienvenidas! Por favor, realizá un fork del proyecto y enviá un pull request.

## Características Recientes (v1.1)

- ✅ Salto directo de página (escribí número + Enter)
- ✅ Navegación con teclado en explorador (flechas + Enter)
- ✅ Backspace para subir directorios (límite configurable)
- ✅ Scroll con flechas Arriba/Abajo (igual que mouse wheel)
- ✅ Zoom optimizado para CBZ/CBR (sin regenerar texturas)
- ✅ Soporte para archivos con `#` y otros caracteres especiales
- ✅ Directorios temporales únicos para CBR (hash de ruta)
- ✅ Limpieza de código (eliminación de duplicación)
