# TracerCube

Raytracer sencillo escrito en Rust que renderiza un cubo sin texturas. El cubo
usa un color base y solamente iluminación difusa de Lambert:

```text
intensidad = max(normal · dirección_hacia_la_luz, 0)
color_final = color_base * intensidad
```

## Ejecutar

```powershell
cargo run --release
```

Presiona `Esc` para cerrar la ventana.

