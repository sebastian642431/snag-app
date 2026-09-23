# Snag

Descargador de audio y video, construido sobre yt-dlp. Hay dos versiones de la misma app, con la misma logica adentro y distinta tecnologia de interfaz.

## Instalar

Doble click en el instalador que quieras. Hace todo solo:

**`Install-Snag-egui.exe`** — la liviana. Un solo proceso, ~70 MB de RAM. Es la que conviene para uso diario.

**`Install-Snag-tauri.exe`** — la de interfaz web. Se ve casi igual pero usa ~369 MB repartidos en 7 procesos, porque levanta el motor de Chrome por debajo.

Podes instalar las dos a la vez sin que se pisen: cada una va a su propia carpeta y tiene su propio acceso directo.

Cada instalador copia la app a `%LOCALAPPDATA%\Programs\`, crea el acceso directo en el Escritorio y en el menu Inicio, y la registra en **Agregar o quitar programas**. No pide permisos de administrador ni toca nada del sistema.

Para saber cual estas usando, mirala en la barra de titulo: dice **Snag · egui** o **Snag · Tauri**.

## Sin ventana

**`Snag-console.cmd`** — la version de terminal, por si no queres interfaz. Te pide el link, elegis 1 (MP3) o 2 (video) y listo. No se instala, se ejecuta directo.

## Desinstalar

Por **Agregar o quitar programas** de Windows, como cualquier otro programa. O ejecutando `Uninstall.exe` que queda en la carpeta de instalacion.

Borra la app, los accesos directos y el registro. No toca los archivos que descargaste.

## Que necesita instalado

- **yt-dlp** — es el que descarga. `winget install yt-dlp.yt-dlp`
- **ffmpeg** — convierte a MP3 y une video con audio. `winget install Gyan.FFmpeg`

No hace falta abrir una terminal: el panel **Tools** de la ventana los instala con winget si faltan, te muestra la version de cada uno y los actualiza de a uno o los dos juntos con **Update all**. La app los busca sola en el PATH y, si no estan, en las carpetas de winget. Los dos puntitos de abajo muestran la version instalada: verde es que esta, rojo es que falta.

Los archivos descargados van a `Downloads\yt-dlp`, y podes cambiar esa carpeta desde la app.

## Carpetas

| Carpeta | Que hay |
| --- | --- |
| `core\` | Todo lo que las dos apps comparten: encontrar las herramientas, los ajustes, el parseo, y correr winget, yt-dlp y la busqueda de actualizaciones. Aca viven los tests |
| `app-egui\` | Codigo de la version liviana. Rust + egui |
| `app-tauri\` | Codigo de la version web. Rust + Tauri, interfaz en HTML/CSS |
| `installer\` | Codigo del instalador. Se compila una vez por cada app, con la app embebida adentro |

## Recompilar

Es un workspace: se compila todo desde la raiz y comparten un unico `target\`.

```
cargo build --release -p snag-egui -p snag-tauri
```

La primera vez tarda unos minutos porque compila las dependencias. Despues es rapido.

Para rearmar un instalador, siempre desde la raiz:

```
$env:PAYLOAD_EXE  = "$PWD\target\release\snag-egui.exe"
$env:PRODUCT_NAME = 'Snag egui'
$env:EXE_NAME     = 'Snag-egui.exe'
$env:APP_VERSION  = '1.0.0'
cargo build --release -p snag-installer
```

Queda en `target\release\snag-setup.exe` y se copia a la raiz como `Install-Snag-egui.exe`. Para la version Tauri es igual cambiando los tres valores.

## Antes de subir cambios

Lo mismo que va a correr GitHub. Si esto pasa, el CI pasa:

```
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Publicar una version

La version esta en un solo lugar: el `Cargo.toml` de la raiz. Los cuatro crates la heredan.

La cambias ahi, commiteas, y despues:

```
git tag v1.0.1
git push origin v1.0.1
```

El tag dispara el build, que se niega a seguir si el tag no coincide con la version del workspace. Si pasa, crea el release y le adjunta los dos instaladores solo.

**Importante:** si movés o renombrás una de estas carpetas, corré `cargo clean` adentro antes de volver a compilar. El compilador guarda rutas absolutas y falla si la carpeta cambió de sitio.

## Espacio

Las carpetas `target\` son del compilador y se regeneran solas. Ocupan varios GB. Para recuperar espacio, `cargo clean` dentro de cada una, con VS Code cerrado.
