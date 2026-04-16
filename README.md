# embedded-sqrt

> Racine carrée en virgule fixe Q15 pour systèmes embarqués — `no_std`, zéro dépendance.

**Copyright (C) 2026 Jorge Andre Castro** — Licence GNU GPL v2 ou ultérieure.

---
 
# Update Version 0.2.0
La version introduit #![forbid(unsafe_code)] pour le safe on veut pas de code unsafe.
## Fonctionnalités

- `#![no_std]` — fonctionne sans bibliothèque standard
- Arithmétique entière pure : pas de flottants, pas de `libm`
- Compatible **Raspberry Pi Pico (RP2040)** et **Pico 2 (RP2350)**
- Algorithme **Newton-Raphson** : convergence quadratique en 6 itérations
- Précision : erreur < 150 ULP sur toute la plage Q15

---

## Format Q15

En Q15, un entier `i32` représente un nombre réel dans `[0.0, 1.0[` :

```
valeur_réelle = valeur_i32 / 32768.0
```

| Valeur `i32` | Valeur réelle |
|---|---|
| `0`     | 0.0         |
| `8192`  | 0.25        |
| `16384` | 0.5         |
| `23170` | ≈ 0.707 (1/√2) |
| `32767` | ≈ 1.0       |

---

## Installation

Dans votre `Cargo.toml` :

```toml
[dependencies]
embedded-sqrt = "0.2.0"
```

---

## Utilisation

```rust
use embedded_sqrt::sqrt;

// sqrt(0.25) = 0.5  →  8192 en Q15 → 16384 en Q15
let res = sqrt(8192);
assert!((res - 16384).abs() < 100);

// sqrt(≈1.0) = ≈1.0
let res = sqrt(32767);
assert!((res - 32767).abs() < 150);

// Entrée négative ou nulle → 0
assert_eq!(sqrt(0), 0);
assert_eq!(sqrt(-1), 0);
```

---

## Algorithme

### Newton-Raphson en virgule fixe

La racine carrée est calculée par la méthode de Newton-Raphson :

```
x_{n+1} = (x_n + a / x_n) / 2
```

#### Étapes

1. **Normalisation**  l'entrée `a` est ramenée dans `[0.5, 2.0[` en Q15 par décalages :
   ```
   sqrt(a · 4^n) = 2^n · sqrt(a)
   ```

2. **Estimation initiale** — deux constantes couvrent les deux moitiés de la plage :
   - `[0.5, 1.0[` → `x0 = 0.84` (≈ 27525 en Q15)
   - `[1.0, 2.0[` → `x0 = 1.19` (≈ 39000 en Q15)

3. **6 itérations** la convergence est quadratique (les bits corrects doublent à chaque étape). 6 itérations suffisent pour dépasser 15 bits de précision.

4. **Dénormalisation** le décalage appliqué à l'entrée est annulé sur le résultat.

### Pourquoi pas CORDIC ?

Le CORDIC hyperbolique pour la racine carrée avec l'initialisation
`x₀ = a + 0.25, y₀ = a - 0.25` n'a **pas un gain constant** sur la plage Q15 :
le facteur K_hyp varie selon la valeur d'entrée, ce qui rend la correction impossible
avec une simple constante. Newton-Raphson est plus simple, plus précis, et garanti convergent.

---

## Compatibilité

| Cible | Statut |
|---|---|
| `thumbv6m-none-eabi` (RP2040, Cortex-M0+) | ✅ |
| `thumbv8m.main-none-eabihf` (RP2350, Cortex-M33) | ✅ |
| `thumbv7em-none-eabihf` (STM32, Cortex-M4F) | ✅ |
| tout target `no_std` | ✅ |

La division `i64` est générée en software par le compilateur sur les cibles
sans division hardware 64 bits (ex. RP2040). Le coût est d'environ 200–300 cycles
par appel sur Cortex-M0+.

---

## Configuration recommandée

`.cargo/config.toml` pour Pico (RP2040) :

```toml
[build]
target = "thumbv6m-none-eabi"
```

Pour Pico 2 (RP2350) :

```toml
[build]
target = "thumbv8m.main-none-eabihf"
```

`Cargo.toml` :

```toml
[profile.release]
opt-level = 3
lto = true
codegen-units = 1
```

---

## Licence

Ce programme est un logiciel libre : vous pouvez le redistribuer et/ou le modifier
selon les termes de la **Licence Publique Générale GNU** telle que publiée par la
Free Software Foundation, soit la version 2 de la licence, soit (à votre choix)
n'importe quelle version ultérieure.

Ce programme est distribué dans l'espoir qu'il sera utile, mais **sans aucune garantie**.

Voir le fichier `LICENSE` ou [gnu.org/licenses](https://www.gnu.org/licenses/) pour les détails.