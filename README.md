# Escena cósmica

Raytracer en Rust con las tres esferas originales:

- Vidrio azulado con reflexión, refracción y efecto Fresnel.
- Cerámica con patrón ajedrezado rojo y dorado.
- Metal naranja cepillado.

El suelo representa un agujero negro procedural con horizonte de eventos, anillo
de fotones, disco de acreción en espiral y polvo cósmico. El cielo conserva sus
estrellas, nebulosa y planeta anillado. Todo se genera matemáticamente, sin usar
archivos de imagen externos.

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
