# Proyecto 2 - Graficas por Computadora

Raytracer CPU escrito en Rust que renderiza un diorama interactivo inspirado en *Angry Birds Space*. La aplicacion abre en pantalla completa con `raylib`, muestra un selector de mundos y permite entrar a cada planeta para explorarlo con una camara orbital.

Todo el raytracing (intersecciones, iluminacion, sombras, reflexion, refraccion y skybox) esta implementado dentro del proyecto. `raylib` solo se usa para la ventana, la entrada, mostrar el framebuffer como textura y dibujar la interfaz 2D.

El proyecto empezo como un diorama de una sala de cine. Esa escena se conserva en `src/cinema.rs` y la usan las pruebas del renderer, pero la aplicacion ya no la muestra.

## Mundos

| Nivel | Mundo | Contenido |
| --- | --- | --- |
| 1 | Luna Azul | Planeta texturizado con crateres, piedras, base de tierra y estructuras sobre la superficie |
| 2 | Mundo Galleta | Planeta galleta con chispas de chocolate, construccion de madera, hielo, metal y TNT, dos cerditos y dulces |
| 3 | Cinturon de Asteroides | Nivel provisional: planeta con asteroides decorativos |

Cada planeta esta rodeado por un campo de gravedad translucido que usa transparencia y refraccion.

## Caracteristicas

- Renderizado por raytracing sobre CPU, paralelizado por filas con `rayon`.
- Primitivas: cubos alineados a ejes, esferas con UV equirectangulares, cajas orientadas, cilindros y conos orientados, unificadas en `Primitive`.
- Marcos radiales para colocar objetos sobre la superficie de un planeta.
- Sombreado local tipo Phong: emision, ambiente, difuso, especular, atenuacion por distancia y sombras duras.
- Reflexion y refraccion recursivas con profundidad maxima de 4, incluida reflexion interna total.
- Materiales con albedo, textura, brillo especular, reflectividad, transparencia, indice de refraccion y emision.
- Texturas PPM `P3` con modos `Repeat` y `Clamp`, y color fallback magenta para texturas invalidas.
- Skybox espacial generado proceduralmente (gradiente, sol, nubes, estrellas y asteroides de fondo).
- Selector de mundos por ray picking: al pasar el mouse sobre un planeta se resalta con un halo y su nombre.
- Render adaptativo: resolucion reducida mientras la camara se mueve y render completo 180 ms despues de detenerse. Solo se vuelve a renderizar cuando cambia la camara o la escena.
- Pruebas unitarias para los modulos principales.

## Requisitos

- Rust y Cargo (edicion 2024).
- CMake y un compilador de C, porque `raylib-sys` compila raylib desde el codigo fuente.
- En Linux, ademas, los paquetes de desarrollo de X11, OpenGL y ALSA:

```bash
sudo apt install cmake libx11-dev libxrandr-dev libxinerama-dev libxcursor-dev libxi-dev libgl1-mesa-dev libasound2-dev
```

Para instalar Rust se puede usar `rustup`:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

## Ejecucion

Desde la raiz del proyecto:

```bash
cargo run --release
```

Se recomienda `--release`: en modo debug el raytracing es mucho mas lento.

La aplicacion abre en pantalla completa con el titulo `Angry Birds Space Diorama - Worlds` y empieza en el selector de mundos. En consola imprime los controles, el numero de threads de `rayon` y, por cada render, la escena, la calidad, la resolucion, el conteo de objetos y la duracion.

## Controles

| Entrada | Accion |
| --- | --- |
| Click izquierdo sobre un planeta | Entrar al nivel (en el selector) |
| Click derecho + arrastrar | Rotar la vista del selector |
| Click izquierdo + arrastrar | Rotar la camara alrededor del planeta (en un nivel) |
| `W` / `S` | Inclinar la camara |
| `A` / `D` | Rotar la camara |
| `Q` / `E` | Acercar o alejar |
| Rueda del mouse | Zoom |
| `R` | Reiniciar la camara |
| `Backspace` | Regresar al selector |
| `Escape` | Salir |

## Resolucion de render

La ventana usa la resolucion del monitor.

| Escena | Render completo | Render interactivo |
| --- | --- | --- |
| Selector | Resolucion de la pantalla | 50 % de la pantalla |
| Niveles | Hasta 1280x720 | Hasta 480x270 |

Cuando el render es menor que la pantalla, se escala con nearest-neighbor.

## Pruebas y formato

Las pruebas no necesitan ventana grafica.

```bash
cargo test
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
```

## Estructura del proyecto

```text
.
├── Cargo.toml
├── Cargo.lock
├── assets/
│   ├── textures/
│   │   ├── blue_moon_planet.ppm        # Textura del planeta Luna Azul
│   │   ├── brushed_metal.ppm           # Texturas de la escena de cine
│   │   ├── demo_checker.ppm
│   │   ├── night_cinema_skybox.ppm
│   │   ├── popcorn_cardboard.ppm
│   │   ├── seat_fabric.ppm
│   │   ├── theater_carpet.ppm
│   │   └── transparent_plastic.ppm
│   ├── Planets/                        # Sprites de referencia (no versionados)
│   └── Background_Elements/            # Fondos de referencia (no versionados)
└── src/
    ├── app.rs           # Ventana raylib, ciclo principal, selector, controles, interfaz y render adaptativo
    ├── basis.rs         # Bases ortonormales para primitivas orientadas
    ├── camera.rs        # Camara pinhole y camara orbital
    ├── cinema.rs        # Escena de la sala de cine (usada en pruebas)
    ├── color.rs         # Operaciones con color RGB normalizado
    ├── cone.rs          # Conos orientados
    ├── cube.rs          # Cubos alineados a ejes
    ├── cylinder.rs      # Cilindros orientados
    ├── framebuffer.rs   # Buffer de pixeles y escalado nearest-neighbor
    ├── intersection.rs  # Datos de impacto de rayos
    ├── lib.rs           # Modulos publicos del crate
    ├── light.rs         # Luces puntuales
    ├── main.rs          # Punto de entrada
    ├── material.rs      # Definicion de materiales
    ├── math.rs          # Vectores y operaciones matematicas
    ├── oriented_box.rs  # Cajas orientadas
    ├── primitive.rs     # Enum que unifica las primitivas
    ├── radial.rs        # Marcos radiales para colocar objetos sobre un planeta
    ├── ray.rs           # Rayos y evaluacion parametrica
    ├── renderer.rs      # Raytracing, sombreado, sombras, reflexion y refraccion
    ├── scene.rs         # Contenedor de primitivas, materiales, texturas, luces y skybox
    ├── skybox.rs        # Muestreo de fondo equirectangular
    ├── space.rs         # Selector de mundos, niveles, materiales, luces y skybox espacial
    ├── sphere.rs        # Esferas con UV equirectangulares
    └── texture.rs       # Carga y muestreo de texturas PPM
```

## Texturas y assets

Las texturas se cargan desde rutas relativas en `assets/textures/`, por lo que el proyecto debe ejecutarse desde la raiz. Si falta una textura requerida, la construccion de la escena devuelve un error.

El skybox espacial no se carga de un archivo: se genera en codigo en `src/space.rs`.

### Atribucion

*Angry Birds Space* es propiedad de Rovio Entertainment. La textura `blue_moon_planet.ppm` se derivo de un sprite de planeta del juego y se usa solo con fines academicos. Los sprites originales (`assets/Planets/` y `assets/Background_Elements/`) se usan como referencia local y no se incluyen en el repositorio.

## Notas de implementacion

`src/main.rs` llama a `app::run()`, que abre la ventana y construye la escena del selector con `space::build_galaxy_selector_scene()`. Al hacer click sobre un planeta, la aplicacion construye la escena del nivel (`build_blue_moon_scene`, `build_cookie_world_scene` o `build_level_three_scene`) y su camara orbital. Con `Backspace` vuelve a construir el selector.

El renderizado principal esta en `src/renderer.rs`. Para cada pixel genera un rayo desde la camara, busca la interseccion mas cercana y calcula el color combinando emision, ambiente, difuso, especular, sombras, reflexion, refraccion y el skybox.

## Dependencias

- `raylib` 6.0: ventana, entrada, textura para mostrar el framebuffer y textos de la interfaz.
- `rayon` 1.10: paralelizacion del render por filas del framebuffer.
