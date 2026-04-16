// Copyright (C) 2026 Jorge Andre Castro
//
// Ce programme est un logiciel libre : vous pouvez le redistribuer et/ou le modifier
// selon les termes de la Licence Publique Générale GNU telle que publiée par la
// Free Software Foundation, soit la version 2 de la licence, soit (à votre convention)
// n'importe quelle version ultérieure.

//! # embedded-sqrt
//!
//! Racine carrée en virgule fixe Q15 pour systèmes embarqués.
//!
//! ## Caractéristiques
//!
//! - `#![no_std]` — aucune dépendance à la bibliothèque standard
//! - Arithmétique entière pure (pas de flottants, pas de `libm`)
//! - Compatible RP2040 (Cortex-M0+) et RP2350 (Cortex-M33)
//! - Algorithme Newton-Raphson (convergence quadratique, 6 itérations)
//!
//! ## Format Q15
//!
//! En Q15, un `i32` représente un nombre réel dans `[0.0, 1.0[` :
//! ```text
//! valeur_réelle = valeur_i32 / 32768.0
//! ```
//! Exemples :
//! - `0`     → 0.0
//! - `8192`  → 0.25
//! - `16384` → 0.5
//! - `23170` → 0.707 (≈ 1/√2)
//! - `32767` → ≈ 1.0
//!
//! ## Exemple
//!
//! ```rust
//! use embedded_sqrt::sqrt;
//!
//! // sqrt(0.25) = 0.5
//! assert_eq!(sqrt(8192), 16384);
//!
//! // sqrt(0.0) = 0.0
//! assert_eq!(sqrt(0), 0);
//! ```

#![no_std]
#![forbid(unsafe_code)]

/// Calcule la racine carrée d'un nombre en virgule fixe Q15.
///
/// # Arguments
///
/// * `a` — valeur en Q15 dans `[0, 32767]`
///   (`valeur_réelle = a / 32768.0`)
///
/// # Retour
///
/// `sqrt(a)` en Q15. Retourne `0` si `a <= 0`.
///
/// # Précision
///
/// Erreur maximale < 150 ULP (unités au dernier rang) sur toute la plage,
/// soit < 0.005 en valeur réelle.
///
/// # Algorithme
///
/// Newton-Raphson en arithmétique entière Q15 :
/// ```text
/// x_{n+1} = (x_n + a / x_n) / 2
/// ```
/// L'entrée est normalisée dans `[0.5, 2.0[` avant l'itération,
/// puis dénormalisée via `sqrt(a · 4^n) = 2^n · sqrt(a)`.
///
/// # Exemples
///
/// ```rust
/// use embedded_sqrt::sqrt;
///
/// // sqrt(0.25) = 0.5  →  8192 → 16384
/// assert!((sqrt(8192) - 16384).abs() < 100);
///
/// // sqrt(≈1.0) = ≈1.0  →  32767 → 32767
/// assert!((sqrt(32767) - 32767).abs() < 150);
///
/// // valeurs négatives → 0
/// assert_eq!(sqrt(-1), 0);
/// ```
pub fn sqrt(a: i32) -> i32 {
    if a <= 0 { return 0; }

    //  Normalisation 
    // On ramène val dans [16384, 65536[ = [0.5, 2.0[ en Q15.
    // Propriété utilisée : sqrt(a · 4^n) = 2^n · sqrt(a)
    // On mémorise n dans `shift` pour dénormaliser le résultat.
    let mut val = a as i64;
    let mut shift = 0i32;
    while val < 16384 {
        val <<= 2;
        shift += 1;
    }
    while val >= 65536 {
        val >>= 2;
        shift -= 1;
    }

    //  Estimation initiale 
    // Deux points de départ couvrant [0.5, 1.0[ et [1.0, 2.0[.
    // Erreur initiale < 20 %, suffisante pour converger en 6 itérations.
    let mut x: i64 = if val < 32768 {
        27525 // ≈ 0.84 en Q15, centre géométrique de [√0.5, 1.0[
    } else {
        39000 // ≈ 1.19 en Q15, centre géométrique de [1.0, √2[
    };

    //  Itérations Newton-Raphson 
    // x_{n+1} = (x_n + val / x_n) / 2
    //
    // Division Q15 : (val / x) en Q15 = (val << 15) / x
    // La convergence est quadratique : les bits corrects doublent à chaque étape.
    // 6 itérations donnent > 15 bits de précision.
    for _ in 0..6 {
        let div = (val << 15) / x;
        x = (x + div) >> 1;
    }

    //  Dénormalisation 
    // On annule le shift appliqué à l'entrée.
    if shift >= 0 {
        (x >> shift) as i32
    } else {
        (x << (-shift)) as i32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sqrt_perfect_square() {
        // sqrt(0.25) = 0.5 → sqrt(8192) = 16384 en Q15
        let res = sqrt(8192);
        assert!((res - 16384).abs() < 100, "Reçu: {}", res);
    }

    #[test]
    fn test_sqrt_one() {
        // sqrt(≈1.0) = ≈1.0 → sqrt(32767) = 32767 en Q15
        let res = sqrt(32767);
        assert!((res - 32767).abs() < 150, "Reçu: {}", res);
    }

    #[test]
    fn test_sqrt_various_values() {
        // sqrt(0.09) = 0.3 → sqrt(2949) = 9830 en Q15
        let res = sqrt(2949);
        assert!((res - 9830).abs() < 100, "Reçu: {}", res);
    }

    #[test]
    fn test_sqrt_zero_and_negative() {
        assert_eq!(sqrt(0), 0);
        assert_eq!(sqrt(-1), 0);
    }
}