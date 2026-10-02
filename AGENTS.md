# AGENTS.md

## Proyecto

Este repositorio contiene un raytracer CPU escrito en Rust que representa un
diorama interactivo inspirado en Angry Birds Space: un selector de mundos y
cuatro niveles (Luna Azul, Mundo Galleta, Cinturon de Asteroides y Cristales
Cosmicos).

- raylib se usa para ventana, entrada, audio, mostrar el framebuffer como
  textura y dibujar la interfaz 2D (textos, paneles, halos del selector e
  imagenes de referencia).
- El raytracing, intersecciones, iluminacion y optica se implementan dentro del
  proyecto. No usar el render 3D, modelos ni shaders de raylib para la escena.
- Rayon se utiliza para paralelizar el calculo del framebuffer en CPU.
- No sustituir el renderer por un motor grafico, una libreria de raytracing o una
  implementacion GPU salvo autorizacion explicita.
- Para ejecucion y mediciones de rendimiento usar `cargo run --release`.

La escena original de la sala de cine (`src/cinema.rs`) ya no se muestra en la
aplicacion, pero se conserva y la usan las pruebas del renderer.

## Sistema de coordenadas

Escenas espaciales, segun `src/space.rs`:

- Y representa altura.
- Los planetas tienen centros y radios definidos como constantes; el cuarto
  nivel contiene varios cuerpos con centros y radios propios.
- Los objetos sobre un planeta se colocan con `RadialFrame` (`src/radial.rs`):
  un marco local con eje radial hacia afuera y dos ejes tangentes.
- Las posiciones del selector usan las constantes `GALAXY_SELECTOR_*`.

Escena de cine (legado), segun `src/cinema.rs`:

- X representa izquierda y derecha, Y altura y Z profundidad.
- La pantalla se encuentra hacia Z negativo y la apertura hacia Z positivo.
- Constantes: `ROOM_WIDTH = 8.0`, `ROOM_HEIGHT = 4.2`, `ROOM_DEPTH = 11.2`,
  `FLOOR_Y = -1.2` y `SCREEN_Z = -5.92`.

## Arquitectura

Flujo real del programa:

- `src/main.rs` llama a `app::run()`.
- `src/app.rs` gestiona ventana raylib, entrada, audio, estados
  (`SceneState::Galaxy` y `SceneState::Planet`), ray picking del selector,
  visor de referencias, interfaz 2D y render adaptativo.
- `src/space.rs` y `src/space/` construyen el selector, los cuatro niveles,
  sus materiales, luces, camaras orbitales y el skybox espacial procedural.
- `src/game.rs` contiene la logica jugable del primer nivel.
- `src/cinema.rs` construye la escena legado de la sala de cine.
- `src/renderer.rs` genera rayos y calcula el color mediante `trace_ray`.
- `src/scene.rs` almacena primitivas, materiales, texturas, luces y skybox.
- `src/primitive.rs` unifica las primitivas en el enum `Primitive`.
- `src/cube.rs`, `src/sphere.rs`, `src/oriented_box.rs`, `src/cylinder.rs` y
  `src/cone.rs` implementan las intersecciones.
- `src/basis.rs` define bases ortonormales para primitivas orientadas.
- `src/radial.rs` define marcos radiales sobre la superficie de un planeta.
- `src/camera.rs` implementa la camara pinhole y la camara orbital.
- `src/framebuffer.rs` almacena pixeles y escalado nearest-neighbor.
- `src/material.rs` y `src/texture.rs` definen materiales y texturas.
- `src/skybox.rs` implementa el entorno equirectangular.

El render paralelo divide el framebuffer por filas mediante Rayon y
`par_chunks_mut`. Existe una referencia secuencial bajo `cfg(test)` para
verificar equivalencia.

Al seleccionar un planeta se reconstruye la escena del nivel. En un nivel, `M`
abre la imagen de referencia correspondiente; el boton `Cerrar` o Backspace la
cierra. Con la referencia cerrada, Backspace reconstruye el selector. Las
escenas no se construyen durante el render.

Las cinco pistas MP3 de `assets/Music/` se cargan al iniciar, se reproducen en
bucle y cambian al cambiar de escena. Cada cuadro actualiza solo el stream
activo. Las cuatro imagenes JPEG de `assets/Levels_images/` tambien se cargan
al iniciar y se dibujan en la interfaz 2D, sin pasar por el raytracer. Se
resuelven con `CARGO_MANIFEST_DIR`.

## Flujo Git obligatorio

- El proyecto utiliza una rama por sprint.
- La rama del sprint actual es `angry-birds-space`.
- No crear una rama diferente para cada paso.
- Cada paso de implementacion debe producir exactamente un commit local, salvo
  que el usuario solicite explicitamente una auditoria sin commits.
- Antes de modificar:
  - Confirmar rama.
  - Confirmar `git status`.
  - Ejecutar pruebas de linea base.
- Despues de modificar:
  - Ejecutar validaciones.
  - Revisar el diff.
  - Crear unicamente el commit solicitado.
  - Confirmar repositorio limpio.
  - Detenerse para revision manual.
- No hacer push, pull, fetch, merge, rebase, reset, stash o tag salvo peticion
  explicita.
- No usar comandos destructivos.
- No sobrescribir cambios existentes del usuario.
- Las auditorias read-only no modifican archivos ni generan commits. En ellas,
  usar `git --no-optional-locks` para que Git no escriba en `.git`.
- `main` contiene los sprints integrados hasta `cine`.
- Si Git se ejecuta desde Linux sobre esta carpeta de Windows, puede marcar
  todos los archivos como modificados por diferencias CRLF/LF. Usar
  `git diff --ignore-cr-at-eol` para ver los cambios reales y hacer los commits
  desde Windows.

## Validacion obligatoria

Antes de cada cambio:

- `git branch --show-current`
- `git status --short`
- `cargo test`

Despues de cada cambio:

- `cargo fmt`
- `cargo fmt --check`
- `cargo test`
- `cargo test --release`
- `cargo check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `git diff --check`
- `git status --short`

Si Clippy es bloqueado por Windows Application Control:

- Reportar el comando y error exactos.
- No desactivar Clippy.
- No cambiar politicas del sistema.
- No considerar automaticamente el bloqueo como defecto del codigo.

Para ejecucion visual:

- Usar `cargo run --release`.
- No evaluar rendimiento mediante `cargo run` en debug.
- Si falla por `XOpenDisplay` o falta de display, reportarlo como limitacion.
- No instalar ni levantar servidores graficos automaticamente.

Compilar en Linux requiere CMake, un compilador de C y los paquetes de
desarrollo de X11, OpenGL y ALSA, porque `raylib-sys` compila raylib desde el
codigo fuente. Las pruebas no necesitan display.

## Reglas de implementacion

- Inspeccionar archivos relevantes antes de editar.
- Mantener los cambios dentro del alcance solicitado.
- Usar helpers y constantes con nombres claros.
- Evitar indices magicos.
- Evitar `unwrap`, `expect` y `panic` nuevos sin justificacion.
- No agregar dependencias sin autorizacion.
- No cargar archivos durante el render.
- Mantener el audio y las imagenes de referencia fuera del camino de trazado
  de rayos. Al mostrar la referencia, pausar la interaccion del nivel y de la
  camara; Backspace debe cerrar el visor antes de regresar al selector.
- No clonar `Scene`, `Material` o `Texture` por pixel o rayo.
- No crear colecciones dentro de `trace_ray`.
- No introducir `Mutex` o `RwLock` por pixel.
- No usar `unsafe` para forzar `Send` o `Sync`.
- Mantener `material_id` y `texture_id` validos.
- Evitar geometria degenerada.
- Evitar superficies coplanares que produzcan z-fighting.
- Mantener posiciones, dimensiones y colores finitos.
- No usar rutas absolutas dependientes de una computadora.
- Las pruebas de `src/space.rs` verifican conteos de objetos mediante
  `SpaceSceneMetadata`. Al agregar objetos a un nivel, esos contadores y sus
  pruebas tambien cambian.

## Render y rendimiento

- El renderer se ejecuta en CPU.
- La GPU no acelera directamente esta implementacion.
- La aplicacion abre en pantalla completa y el framebuffer usa la resolucion
  del monitor.
- Selector: render completo a resolucion de pantalla e interactivo a escala 0.5.
- Niveles: render completo limitado a 1280x720 e interactivo limitado a 480x270.
- El retardo antes del render completo es 180 ms.
- La profundidad maxima de recursion es 4 (`MAX_RECURSION_DEPTH`).
- Conservar el render bajo demanda.
- No renderizar nuevamente cuando camara y escena no cambian.
- Conservar el escalado nearest-neighbor y el filtro point de la textura raylib.
- Conservar la particion paralela mediante `par_chunks_mut`.
- `Scene` y `Camera` deben compartirse mediante referencias inmutables.
- No agregar bloqueos al camino caliente.
- Mantener compatibilidad con `RAYON_NUM_THREADS=1`.
- No cambiar resolucion, recursion o calidad para ocultar problemas sin permiso.
- No implementar BVH sin medir primero el rendimiento real.
- No usar tiempos absolutos como assertions de pruebas.

## Funcionalidad visual

Conservar:

- Selector de mundos con ray picking, halo al pasar el mouse y nombre del nivel.
- Tema musical del selector y una pista en bucle por nivel.
- Visor de referencia por nivel con `M`, boton de cierre y Backspace.
- Regreso al selector con Backspace.
- Camara orbital: rotacion con teclado y arrastre del mouse, zoom con teclado y
  rueda, y reset.
- Iluminacion ambiental, difusa y especular.
- Atenuacion.
- Sombras.
- Reflexion recursiva.
- Refraccion.
- Reflexion interna total.
- Skybox espacial procedural.
- Fondo fallback.
- Render adaptativo.
- Render paralelo.
- Overlays de estado y controles.

Contenido actual de las escenas espaciales:

- Selector: cuatro planetas con campo de gravedad translucido.
- Luna Azul: primer nivel jugable con pajaros, resortera, gravedad, puntaje y
  trayectoria. El diorama anterior de Luna Azul sigue disponible para pruebas.
- Mundo Galleta: planeta galleta, chispas de chocolate, construccion de madera,
  hielo, metal y TNT, dos cerditos y dulces.
- Cinturon de Asteroides: puente de asteroides, resortera, pajaros, cerditos y
  bloques.
- Cristales Cosmicos: cuarto escenario espacial.

Escena de cine (legado): mantener los cinco materiales texturizados (tela,
alfombra, metal, plastico transparente y carton de palomitas) y los elementos
que verifican sus pruebas.

## Pruebas

- Todas las pruebas anteriores deben continuar pasando.
- La linea base verificada el 2026-10-01 es de 722 pruebas.
- Agregar pruebas para cambios funcionales.
- No eliminar, ignorar o relajar pruebas para hacerlas pasar.
- Evitar pruebas dependientes de indices absolutos.
- Preferir constantes, helpers y metadatos.
- Usar tolerancias adecuadas para flotantes.
- No depender de ventana grafica.
- No usar limites de tiempo como assertions.
- Mantener equivalencia entre render paralelo y secuencial.

## Restricciones de alcance

No mezclar en un mismo paso, salvo autorizacion:

- Nuevos objetos de escena.
- Optimizaciones estructurales.
- Cambios opticos.
- Documentacion.
- Cambios de dependencias.
- Cambios de flujo Git.

No actualizar README en pasos que no sean de documentacion.
No agregar assets descargados sin autorizacion y atribucion.

`assets/Planets/` y `assets/Background_Elements/` contienen sprites de Angry
Birds Space usados como referencia local y estan en `.gitignore`.
`assets/textures/blue_moon_planet.ppm` se derivo de uno de esos sprites; su
origen esta indicado en el encabezado del PPM y en la seccion de atribucion del
README.

`assets/Music/` contiene `ABS_PG2_Main_Theme.mp3` y
`ABS_PG2_Level_1.mp3` a `ABS_PG2_Level_4.mp3`. `assets/Levels_images/`
contiene `level_1.jpeg` a `level_4.jpeg`. Ambos directorios estan versionados.

## Estado verificado

Fecha: 2026-10-01. Rama `angry-birds-space`. Antes de este paso de
documentacion, `cargo test` y `cargo test --release` aprobaron 722 pruebas;
`cargo fmt --check`, `cargo check` y Clippy tambien aprobaron en Windows.
Este estado puede quedar desactualizado: verificar siempre Git, codigo y
pruebas antes de modificar.
