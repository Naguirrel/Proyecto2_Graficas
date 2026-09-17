# Proyecto 2 - Graficas por Computadora

Raytracer CPU escrito en Rust para renderizar un diorama con cubos, materiales texturizados, iluminacion local y camara orbital interactiva. La aplicacion abre una ventana de `800x600` usando `minifb` y dibuja una escena con texturas PPM para representar tela, alfombra, metal, plastico transparente y carton.

## Caracteristicas

- Renderizado por raytracing sobre CPU.
- Interseccion de rayos con cubos.
- Camara orbital interactiva.
- Materiales con albedo, textura, brillo especular, reflectividad, transparencia, indice de refraccion y emision.
- Iluminacion ambiental y luces puntuales.
- Sombreado local tipo Phong con sombras duras.
- Carga de texturas PPM en formato `P3`.
- Modos de muestreo de textura `Repeat` y `Clamp`.
- Pruebas unitarias para modulos principales.

## Requisitos

- Rust y Cargo instalados.
- Un entorno grafico compatible con ventanas de escritorio, necesario para `minifb`.

Para instalar Rust se puede usar `rustup`:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

## Ejecucion

Desde la raiz del proyecto:

```bash
cargo run
```

Al iniciar, el programa renderiza la escena y muestra una ventana titulada:

```text
Diorama Raytracing - Materiales texturizados
```

## Controles

| Tecla | Accion |
| --- | --- |
| `W` / `S` | Inclinar la camara hacia arriba o abajo |
| `A` / `D` | Rotar la camara alrededor de la escena |
| `Q` / `E` | Acercar o alejar la camara |
| Rueda del mouse | Zoom |
| `R` | Reiniciar la camara |
| `Escape` | Salir |

## Pruebas

Para ejecutar las pruebas unitarias:

```bash
cargo test
```

Para revisar formato del codigo:

```bash
cargo fmt --check
```

## Estructura del proyecto

```text
.
├── Cargo.toml
├── assets/
│   └── textures/
│       ├── brushed_metal.ppm
│       ├── popcorn_cardboard.ppm
│       ├── seat_fabric.ppm
│       ├── theater_carpet.ppm
│       └── transparent_plastic.ppm
└── src/
    ├── app.rs           # Ventana, ciclo principal y controles
    ├── camera.rs        # Camara pinhole y camara orbital
    ├── color.rs         # Operaciones con color RGB normalizado
    ├── cube.rs          # Geometria e interseccion con cubos
    ├── framebuffer.rs   # Buffer de pixeles
    ├── light.rs         # Luces puntuales
    ├── material.rs      # Definicion de materiales
    ├── math.rs          # Vectores y operaciones matematicas
    ├── renderer.rs      # Raytracing, sombreado y escena de ejemplo
    ├── scene.rs         # Contenedor de objetos, materiales, texturas y luces
    └── texture.rs       # Carga y muestreo de texturas PPM
```

## Texturas

Las texturas se cargan desde `assets/textures/` mediante rutas relativas, por lo que se recomienda ejecutar el proyecto desde la raiz. Si alguna textura no puede cargarse, el renderizador usa una textura fallback magenta para hacer visible el problema.

## Notas de implementacion

La escena de ejemplo se construye en `src/renderer.rs` dentro de `sample_scene()`. Ahi se registran las texturas, materiales, cubos y luces. El renderizado principal recorre cada pixel del framebuffer, genera un rayo desde la camara, busca la interseccion mas cercana en la escena y calcula el color final con iluminacion ambiental, difusa, especular y sombras.

## Dependencias

La dependencia externa principal es:

- `minifb`: creacion de ventana y actualizacion del framebuffer.
