# Proyecto 2 - Graficas por Computadora

Raytracer CPU escrito en Rust que renderiza un diorama interactivo inspirado en *Angry Birds Space*. La aplicacion abre en pantalla completa con `raylib`, muestra un selector de mundos y permite entrar a cada planeta para explorarlo con una camara orbital.

Todo el raytracing (intersecciones, iluminacion, sombras, reflexion, refraccion y skybox) esta implementado dentro del proyecto. `raylib` se usa para la ventana, la entrada, mostrar el framebuffer como textura, dibujar la interfaz 2D y reproducir la musica.

El proyecto empezo como un diorama de una sala de cine. Esa escena se conserva en `src/cinema.rs` y la usan las pruebas del renderer, pero la aplicacion ya no la muestra.

## Mundos

| Nivel | Mundo | Contenido |
| --- | --- | --- |
| 1 | Luna Azul | Nivel jugable de lanzamiento de pajaros con gravedad, trayectoria, puntaje y reinicio |
| 2 | Mundo Galleta | Planeta galleta con chispas de chocolate, construccion de madera, hielo, metal y TNT, dos cerditos y dulces |
| 3 | Cinturon de Asteroides | Puente de asteroides con resortera, pajaros, cerditos y bloques |
| 4 | Cristales Cosmicos | Escenario de cristales cosmicos |

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
- Musica en bucle para el selector y una pista propia para cada nivel.
- Referencia visual de cada nivel accesible con `M`, dibujada como interfaz 2D sobre la escena.
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
| `M` (en un nivel) | Mostrar la imagen de referencia del nivel |
| Boton `Cerrar` o `Backspace` (con la referencia abierta) | Cerrar la referencia y seguir en el nivel |
| `Backspace` (sin referencia abierta) | Regresar al selector |
| Arrastrar y soltar el pajaro (nivel 1) | Apuntar y lanzar |
| `Enter` (nivel 1) | Reiniciar el nivel |
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
│   ├── Music/                          # Tema del selector y pistas de los cuatro niveles
│   ├── Levels_images/                  # Referencias JPEG de los cuatro niveles
│   ├── Planets/                        # Sprites de referencia (no versionados)
│   └── Background_Elements/            # Fondos de referencia (no versionados)
└── src/
    ├── app.rs           # Ventana raylib, audio, visor de referencias, controles y render adaptativo
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
    ├── space.rs         # Selector de mundos y escenas espaciales
    ├── space/           # Geometria y logica de niveles, pajaros y voxeles
    ├── sphere.rs        # Esferas con UV equirectangulares
    └── texture.rs       # Carga y muestreo de texturas PPM
```

## Texturas y assets

Las texturas se cargan desde rutas relativas en `assets/textures/`, por lo que el proyecto debe ejecutarse desde la raiz. Si falta una textura requerida, la construccion de la escena devuelve un error.

La aplicacion carga al iniciar las cinco pistas de `assets/Music/` y las cuatro imagenes de `assets/Levels_images/`. Las rutas se resuelven desde `CARGO_MANIFEST_DIR`; si falta algun archivo, el inicio devuelve un error. La musica se reproduce en bucle y cambia al entrar a un nivel o volver al selector. Mientras la referencia esta abierta, la interaccion del nivel y la camara se pausan.

| Parte | Musica | Referencia |
| --- | --- | --- |
| Selector | `ABS_PG2_Main_Theme.mp3` | — |
| Nivel 1: Luna Azul | `ABS_PG2_Level_1.mp3` | `level_1.jpeg` |
| Nivel 2: Mundo Galleta | `ABS_PG2_Level_2.mp3` | `level_2.jpeg` |
| Nivel 3: Cinturon de Asteroides | `ABS_PG2_Level_3.mp3` | `level_3.jpeg` |
| Nivel 4: Cristales Cosmicos | `ABS_PG2_Level_4.mp3` | `level_4.jpeg` |

El skybox espacial no se carga de un archivo: se genera en codigo en `src/space.rs`.

### Atribucion

*Angry Birds Space* es propiedad de Rovio Entertainment. La textura `blue_moon_planet.ppm` se derivo de un sprite de planeta del juego y se usa solo con fines academicos. Los sprites originales (`assets/Planets/` y `assets/Background_Elements/`) se usan como referencia local y no se incluyen en el repositorio.

## Notas de implementacion

`src/main.rs` llama a `app::run()`, que abre la ventana y construye la escena del selector con `space::build_galaxy_selector_scene()`. Al elegir un planeta, la aplicacion construye el nivel correspondiente y su camara orbital. El nivel 1 usa `space::build_level_one()`; `build_blue_moon_scene()` conserva un diorama anterior para las pruebas. Con `Backspace` se cierra primero la referencia, si esta abierta; una segunda pulsacion vuelve a construir el selector.

El renderizado principal esta en `src/renderer.rs`. Para cada pixel genera un rayo desde la camara, busca la interseccion mas cercana y calcula el color combinando emision, ambiente, difuso, especular, sombras, reflexion, refraccion y el skybox.

## Dependencias

- `raylib` 6.0: ventana, entrada, audio, textura para mostrar el framebuffer, imagenes de referencia y textos de la interfaz.
- `rayon` 1.10: paralelizacion del render por filas del framebuffer.
