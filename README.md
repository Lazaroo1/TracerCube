# TracerCube

Raytracer sencillo escrito en Rust que renderiza dos cubos de distinto color y
tamaño, sin texturas. Puedes moverlos manualmente y ambos usan solamente
iluminación difusa de Lambert:

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

- `1`: seleccionar el cubo naranja.
- `2`: seleccionar el cubo azul.
- `WASD`: mover el cubo seleccionado.
- `Esc`: salir.
