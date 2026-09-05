# TracerCube

Raytracer sencillo escrito en Rust que renderiza dos cubos de distinto color y
tamaño, sin texturas. Los cubos permanecen quietos mientras puedes orbitar la
cámara a su alrededor. Ambos usan solamente iluminación difusa de Lambert:

```text
intensidad = max(normal · dirección_hacia_la_luz, 0)
color_final = color_base * intensidad
```

## Ejecutar

```powershell
cargo run --release
```

Presiona `Esc` para cerrar la ventana.

## Controles

- `A` / `D`: orbitar la cámara hacia la izquierda o derecha.
- `W` / `S`: orbitar la cámara hacia arriba o abajo.
- Flechas: controles alternativos para orbitar.
- `R`: regresar la cámara a su posición inicial.
- `Esc`: salir.
