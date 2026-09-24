# Proyecto 2 - Graficas por Computadora

Raytracer CPU escrito en Rust para renderizar un diorama interactivo de una sala de cine. La aplicacion abre una ventana de `800x600` con `minifb`, construye la escena a partir de cubos y materiales texturizados, y usa una camara orbital para explorar el resultado en tiempo real.

La escena incluye butacas, gradas, pasillo central, pantalla, paredes acusticas, techo parcial, salida iluminada, proyector, skybox nocturno y detalles de una funcion recien terminada como palomitas, vasos, tickets y envoltorios.

## Caracteristicas

- Renderizado por raytracing sobre CPU.
- Paralelizacion del framebuffer con `rayon`.
- Interseccion de rayos con cubos axis-aligned.
- Camara orbital interactiva con renderizado de baja resolucion durante el movimiento y renderizado completo al detenerse.
- Materiales con albedo, textura, brillo especular, reflectividad, transparencia, indice de refraccion y emision.
- Reflexion y refraccion recursivas con limite de profundidad.
- Iluminacion ambiental y luces puntuales.
- Sombreado local tipo Phong con sombras duras.
- Skybox muestreado desde textura PPM.
- Carga de texturas PPM en formato `P3`.
- Modos de muestreo de textura `Repeat` y `Clamp`.
- Pruebas unitarias para los modulos principales.

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

Al iniciar, el programa carga las texturas desde `assets/textures/`, construye la sala de cine y muestra una ventana titulada:

```text
Diorama Raytracing - Sala de cine
```

Tambien imprime en consola los controles, el numero de threads de `rayon` y el tiempo de cada render.

## Controles

| Tecla | Accion |
| --- | --- |
| `W` / `S` | Inclinar la camara hacia arriba o abajo |
| `A` / `D` | Rotar la camara alrededor de la escena |
| `Q` / `E` | Acercar o alejar la camara |
| Rueda del mouse | Zoom |
| `R` | Reiniciar la camara |
| `Escape` | Salir |

## Pruebas y formato

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
├── Cargo.lock
├── assets/
│   └── textures/
│       ├── brushed_metal.ppm
│       ├── demo_checker.ppm
│       ├── night_cinema_skybox.ppm
│       ├── popcorn_cardboard.ppm
│       ├── seat_fabric.ppm
│       ├── theater_carpet.ppm
│       └── transparent_plastic.ppm
└── src/
    ├── app.rs           # Ventana, ciclo principal, controles y calidad interactiva
    ├── camera.rs        # Camara pinhole y camara orbital
    ├── cinema.rs        # Construccion de la sala de cine
    ├── color.rs         # Operaciones con color RGB normalizado
    ├── cube.rs          # Geometria e interseccion con cubos
    ├── framebuffer.rs   # Buffer de pixeles y escalado nearest-neighbor
    ├── intersection.rs  # Datos de impacto de rayos
    ├── lib.rs           # Modulos publicos del crate
    ├── light.rs         # Luces puntuales
    ├── main.rs          # Punto de entrada
    ├── material.rs      # Definicion de materiales
    ├── math.rs          # Vectores y operaciones matematicas
    ├── ray.rs           # Rayos y evaluacion parametrica
    ├── renderer.rs      # Raytracing, sombreado, reflexion y refraccion
    ├── scene.rs         # Contenedor de objetos, materiales, texturas, luces y skybox
    ├── skybox.rs        # Muestreo de fondo panoramico
    └── texture.rs       # Carga y muestreo de texturas PPM
```

## Texturas y assets

Las texturas se cargan desde rutas relativas en `assets/textures/`, por lo que se recomienda ejecutar el proyecto desde la raiz. Si falta una textura requerida por la sala de cine, la aplicacion devuelve un error de carga. El renderizador tambien mantiene un color fallback magenta para hacer visibles referencias de textura invalidas dentro de una escena.

## Notas de implementacion

La escena principal se construye en `src/cinema.rs` mediante `build_cinema_scene()`. Alli se registran texturas, materiales, cubos, luces y skybox. El ciclo de la aplicacion vive en `src/app.rs`: lee la entrada, actualiza la camara orbital y alterna entre render interactivo escalado y render completo.

El renderizado principal esta en `src/renderer.rs`. Para cada pixel genera un rayo desde la camara, busca la interseccion mas cercana en la escena y calcula el color final combinando ambiente, difuso, especular, sombras, reflexion, refraccion y fondo de skybox.

## Dependencias

- `minifb`: creacion de ventana y actualizacion del framebuffer.
- `rayon`: paralelizacion del render por filas del framebuffer.
