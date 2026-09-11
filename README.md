# Escena cósmica

Raytracer en Rust con cuatro esferas en un pequeño sistema planetario:

- Un planeta central con bandas procedurales y anillos tipo Saturno.
- Una esfera de vidrio azulado con reflexión, refracción y efecto Fresnel.
- Una esfera de cerámica con patrón ajedrezado rojo y dorado.
- Una esfera de metal naranja cepillado.

Las tres esferas pequeñas orbitan continuamente alrededor del planeta a radios
y velocidades diferentes, como asteroides o satélites.

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
