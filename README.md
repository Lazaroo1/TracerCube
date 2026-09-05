# TracerCube

Raytracer en Rust con tres cubos idénticos y texturas/materiales procedurales:

- Cerámica con patrón ajedrezado rojo y dorado.
- Metal naranja cepillado.
- Vidrio azulado con reflexión, refracción y efecto Fresnel.

La escena también incluye un suelo ajedrezado, sombras y un cielo en degradado
para que las propiedades reflectantes y transparentes sean visibles. No utiliza
archivos de imagen externos; las texturas se calculan directamente en cada cara.

## Ejecutar

```powershell
cargo run --release
```

## Controles

- `A` / `D`: orbitar la cámara hacia la izquierda o derecha.
- `W` / `S`: orbitar la cámara hacia arriba o abajo.
- Flechas: controles alternativos para orbitar.
- `R`: regresar la cámara a su posición inicial.
- `Esc`: salir.
