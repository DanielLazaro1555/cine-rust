# Reproducción y renderizado de video

Este documento describe cómo Cine conecta la decodificación de `libmpv` con el
renderizado OpenGL de GTK, por qué aparecía tirón durante la reproducción y qué
cambios se hicieron para reducirlo.

## Ruta de un fotograma

La decodificación y la presentación son etapas distintas. `hwdec=auto` puede
delegar la decodificación a la GPU, pero GTK todavía debe recibir y dibujar cada
fotograma a tiempo.

```mermaid
flowchart LR
    A[libmpv decodifica el video] --> B[Callback de actualización de mpv]
    B --> C[Marca un render pendiente]
    C --> D[Reloj de fotogramas de GTK]
    D --> E[GtkGLArea queue_render]
    E --> F[mpv renderiza en el FBO OpenGL de GTK]
    F --> G[GTK presenta el fotograma]
```

La implementación está en `src/mpv_gl_area.rs`. `GtkGLArea` no usa el modo de
renderizado automático: Cine solicita un dibujo cuando `libmpv` notifica una
actualización. `src/mpv_engine.rs` enlaza `mpv_render_context_render()` con el
FBO OpenGL que GTK tiene activo.

## Causa del tirón

La primera ruta enviaba cada notificación de mpv a una tarea `idle` de GLib. Esas
tareas podían esperar detrás de otros eventos antes de pedirle a `GtkGLArea` que
dibujara. Además, `mpv_render_context_render()` puede esperar a la hora de
presentación calculada por mpv mientras se ejecuta en el hilo principal de GTK.
Durante esa espera, el mismo hilo no puede atender otros eventos ni preparar el
siguiente fotograma.

Por eso tener NVDEC activo no bastaba: la GPU podía decodificar correctamente,
pero la solicitud de dibujo y la presentación llegaban tarde. La documentación
de mpv explica que el render API puede bloquear hasta la hora objetivo y que
`video-timing-offset` controla esa espera para clientes que integran el
renderizado. Véase [el manual de mpv](https://mpv.io/manual/master/).

## Cambios aplicados

1. El callback de mpv marca de forma atómica que hay un dibujo pendiente; no
   crea una tarea `idle` por cada actualización.
2. `GtkGLArea` consume esa marca desde `add_tick_callback`, alineando la
   solicitud de dibujo con el reloj de fotogramas de GTK. Las actualizaciones
   repetidas se agrupan en vez de acumular solicitudes atrasadas.
3. Cine establece `video-timing-offset=0` al crear el handle de mpv. GTK ya
   organiza cuándo se dibuja el `GtkGLArea`, así que mpv no necesita mantener
   bloqueado el hilo principal esperando su hora objetivo antes de devolver el
   render.
4. Las métricas locales cuentan avisos, solicitudes, renders, duración de
   `mpv_render_context_render()` y espera hasta el siguiente dibujo. Cada cinco
   segundos también consultan los contadores acumulados de mpv: fotogramas
   descartados por el decodificador y por la salida de video, retrasos y
   fotogramas fuera de tiempo. Las consultas de contadores se hacen al informar,
   no en cada fotograma ni mientras se sostiene el mutex de métricas. Mantener
   ese mutex durante una consulta síncrona a mpv podía bloquear el callback de
   actualización y congelar el render.
5. Al iniciar, Cine captura stdout y stderr —incluidos mensajes de mpv y errores
   de pánico— en un archivo persistente. La ruta predeterminada es
   `~/.local/state/cine/cine.log`; si se define `XDG_STATE_HOME`, se usa
   `$XDG_STATE_HOME/cine/cine.log`. Al superar 5 MiB conserva una rotación en
   `cine.log.1`.

## Medición realizada

La comprobación se hizo reproduciendo el video de prueba H.264, 720×1280, 30 FPS.
mpv confirmó `Using hardware decoding (nvdec)` y la salida `cuda[nv12]`.

| Ruta medida | Renders en 5 s | Tiempo medio del callback de render | Fotogramas descartados por VO reportados |
| --- | ---: | ---: | ---: |
| Solicitud desde tarea `idle` | 27 | 1,38 ms* | 138 acumulados en esa ejecución |
| Reloj de GTK, sin cambiar la espera de mpv | 147 | 25,04 ms* | 5 acumulados en esa ejecución |
| Reloj de GTK, `video-timing-offset=0` y contadores fuera del camino por fotograma | 147–150 | 0,47–0,59 ms | 4 acumulados en esa ejecución |

Los valores de descartes son acumulados desde el inicio de cada reproducción,
no una tasa por intervalo. Las ejecuciones se hicieron por separado y sirven
como evidencia del problema y de la mejora observada, no como benchmark
controlado para todas las GPU, pantallas o videos. El intervalo final alcanzó
30 FPS para la fuente de 30 FPS, con un render medio inferior a 1 ms.

\* En las dos primeras mediciones, el tiempo del callback incluía también cuatro
lecturas síncronas de propiedades de mpv que se hacían en cada fotograma para
reunir estadísticas. Se movieron fuera del camino por fotograma. Como ese cambio
y `video-timing-offset=0` quedaron aplicados antes de la tercera medición, las
cifras demuestran que la ruta combinada mejoró, pero no aíslan cuánto aportó cada
cambio por separado.

## Consideraciones

- El límite de fluidez de esta prueba es la fuente de 30 FPS. Para superar ese
  ritmo habría que interpolar fotogramas, una función distinta que puede añadir
  carga y artefactos.
- La frecuencia del monitor también afecta la cadencia. Una fuente de 30 FPS se
  reparte de forma regular en pantallas de 60 o 120 Hz, pero no necesariamente en
  una pantalla con una frecuencia que no sea múltiplo entero de 30. Eso puede
  sentirse como movimiento irregular incluso cuando el render no tarda y mpv no
  descarta fotogramas. Se añadieron `display-fps`, `estimated-display-fps` y
  `vsync-ratio` a las métricas para comprobar este caso en cada equipo.
- El temporizador de la interfaz consulta posición, duración y pausa cada 250 ms
  para mantener fluido el control de progreso. Las dimensiones y los datos de
  calidad, que cambian mucho menos, se consultan ahora cada segundo. Esto reduce
  las lecturas síncronas aproximadas de 48 a 21 por segundo cuando todas las
  propiedades están disponibles. En la medición anterior a este ajuste, el
  callback completo tardó menos de 0,5 ms de media y menos de 3,5 ms en su peor
  muestra; no parecía ser el cuello de botella principal, pero reducir las
  consultas deja más margen al hilo de GTK. También se registra el intervalo
  máximo entre callbacks para detectar pausas del loop de GTK.
- En una muestra se observó una espera aislada de aproximadamente 205 ms entre
  el aviso de mpv y el render, mientras que la mayoría de las esperas estuvieron
  por debajo de 10 ms. Esa pausa requiere repetir la medición para saber si fue
  un cambio de estado/arranque o un bloqueo real durante reproducción continua.
- `hwdec=auto` confirma que la decodificación usa la GPU, pero no garantiza por
  sí solo una presentación fluida. El compositor, el controlador, el intercambio
  de superficies CUDA/OpenGL y la frecuencia de pantalla forman parte de la ruta.
- `video-timing-offset=0` evita la espera interna que bloqueaba el hilo de GTK.
  Conviene comprobar la sincronía entre audio y video también con clips largos,
  distintas tasas de fotogramas y otras salidas de audio.
- El render OpenGL sigue ejecutándose dentro del callback de dibujo de GTK,
  porque `GtkGLArea` proporciona allí el contexto GL activo. Moverlo a otro hilo
  requeriría diseñar y sincronizar otro contexto OpenGL; no es un cambio seguro
  de una sola línea.
- `cargo check` se ejecutó después de los cambios.
