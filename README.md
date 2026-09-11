# TracerCube

Raytracer en Rust con tres cubos idénticos y texturas/materiales procedurales:

- Cerámica con patrón ajedrezado rojo y dorado.
- Metal naranja cepillado.
- Vidrio azulado con reflexión, refracción y efecto Fresnel.

La escena también incluye un tablero ajedrezado neón con halos de color, sombras,
bordes luminosos en cada cubo y un cielo cósmico con estrellas, nebulosa y un
planeta anillado. No utiliza archivos de imagen externos: todos los detalles se
calculan directamente en el raytracer.

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
