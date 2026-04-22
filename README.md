[![Crates.io](https://img.shields.io/crates/v/embedded-sqrt.svg)](https://crates.io/crates/embedded-sqrt)
[![Docs.rs](https://docs.rs/embedded-sqrt/badge.svg)](https://docs.rs/embedded-sqrt)
[![License: GPL v2](https://img.shields.io/badge/License-GPL_v2-blue.svg)](https://www.gnu.org/licenses/old-licenses/gpl-2.0.en.html)


# embedded-sqrt
> Racine carrée en virgule fixe Q15 pour systèmes embarqués `no_std`, zéro dépendance, testée sur pico 2040 sans fpu.

---
# Update version 0.2.1 testée sur la pico 2040 qui n'a pas de fpu
La version 0.2.1 est identique à la version 0.2.0 , elle introduit un exemple clé en main pour la pico 2040 qui n'a pas de FPU , rétrouvez l'exemple dans la section exemples .

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
|---------|--------------------|
| `0`     | 0.0                |
| `8192`  | 0.25               |
| `16384` | 0.5                |
| `23170` | ≈ 0.707 (1/√2)     |
| `32767` | ≈ 1.0              |

---

## Installation

Dans votre `Cargo.toml` :

```toml
[dependencies]
embedded-sqrt = "0.2.2"
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

# Exemple Pico 2040 sans FPU 

````rust #![no_std]
#![no_main]

use core::fmt::Write;
use cortex_m_rt as _;
use embassy_executor::Spawner;
use embassy_rp::i2c::{Config as I2cConfig, I2c, Async};
use embassy_time::{Delay, Timer};
use {panic_halt as _, embassy_rp as _};

// Mathématiques et formatage
use heapless::String;
use embedded_sqrt::sqrt;

// Drivers et partage de bus
use hd44780_i2c_nostd::LcdI2c;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_embedded_hal::shared_bus::asynch::i2c::I2cDevice;

use embassy_rp::bind_interrupts;
use embassy_rp::peripherals::I2C0; 


// Configuration des interruptions pour l'I2C0
bind_interrupts!(struct Irqs {
    I2C0_IRQ => embassy_rp::i2c::InterruptHandler<I2C0>;
});

// TASK : SYSTEM_TASK
// Cette tâche s'occupe de calculer et d'afficher les racines carrées
#[embassy_executor::task]
async fn system_task(
    mut lcd: LcdI2c<I2cDevice<'static, NoopRawMutex, I2c<'static, I2C0, Async>>>
) {
    let mut delay = Delay;
    
    // Initialisation du LCD via le device partagé
    if let Ok(_) = lcd.init(&mut delay).await {
        let _ = lcd.set_backlight(true);
        let _ = lcd.clear(&mut delay).await;
    }

    let calculs = [4, 16, 25, 23];
    let mut idx = 0;
loop {
    let n = calculs[idx];
    
    // 1. On donne un entier, il nous rend une racine formatée Q15
    let res_q15 = sqrt(n); 

    // 2. On convertit pour l'humain 
    
    let res_humain = (res_q15 as i32 * 181 + 16384) / 32768;

    // 3. Affichage 
    let _ = lcd.clear(&mut delay).await;
    let _ = lcd.set_cursor(0, 0, &mut delay).await;
    let _ = lcd.write_str("MATH Q15 Test :", &mut delay).await;

    let _ = lcd.set_cursor(1, 0, &mut delay).await;
    let mut s: String<20> = String::new();
    let _ = write!(s, "sqrt({}) = {}", n, res_humain);
    let _ = lcd.write_str(s.as_str(), &mut delay).await;

    idx = (idx + 1) % calculs.len();
    Timer::after_secs(2).await;
}

}

// MAIN : POINT D'ENTRÉE 
#[embassy_executor::main]
async fn main(spawner: Spawner) {
    // Initialisation du hardware de la Pico
    let p = embassy_rp::init(embassy_rp::config::Config::default());
    
    // Attente de stabilisation
    Timer::after_millis(500).await;

    // 1. Configuration du bus I2C0 physique (SCL=GP9, SDA=GP8)
    let mut i2c_config = I2cConfig::default();
    i2c_config.frequency = 100_000; 
    let i2c_bus = I2c::new_async(p.I2C0, p.PIN_9, p.PIN_8, Irqs, i2c_config);

    // 2. Création du Mutex statique pour partager le bus
    // On utilise StaticCell pour que l'objet vive pendant toute la durée du programme
    static I2C_BUS: static_cell::StaticCell<Mutex<NoopRawMutex, I2c<'static, I2C0, Async>>> = static_cell::StaticCell::new();
    let i2c_mutex = I2C_BUS.init(Mutex::new(i2c_bus));

    // 3. Création du périphérique virtuel pour le LCD
    let i2c_dev_lcd = I2cDevice::new(i2c_mutex);

    // 4. Création de l'instance du driver LCD
    let lcd = LcdI2c::new(i2c_dev_lcd, 0x3F);

    // 5. Lancement de la tâche asynchrone
    // spawner.spawn(system_task(lcd)).unwrap(); est la clé ici
    if let Err(_) = spawner.spawn(system_task(lcd)) {
        // Erreur si la tâche ne peut pas être lancée
    }

    // Le main a fini son travail d'initialisation, 
    // l'exécuteur Embassy prend le relais pour faire tourner system_task.
}
````

**Le Cargo.toml indispensable sans lui pas de programme ni affichage**
````
[dependencies]
embassy-rp = { version = "0.6.0", features = ["rt", "rp2040", "time-driver", "critical-section-impl"] }
embassy-executor = { version = "0.6.3", features = ["arch-cortex-m", "executor-thread", "task-arena-size-32768"] }
embassy-time = { version = "0.4.0", features = ["generic-queue-8"] }
embassy-sync = { version = "0.6.1" }
embassy-embedded-hal = { version = "0.3.0" }

embedded-hal = "1.0.0"
embedded-hal-async = "1.0.0"
embedded-hal-bus = { version = "0.2.0", features = ["async"] }
portable-atomic = { version = "1.5" }

cortex-m = "0.7.7"
cortex-m-rt = "0.7.3"
panic-halt = "0.2.0"
heapless = "0.8.0"


hd44780-i2c-nostd = "0.3.0"
embedded-sqrt = "0.2.0"
static_cell = "2.1.1"

[profile.release]
opt-level = "s"
lto = true
codegen-units = 1
panic = "abort"
strip = true
````

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

**Copyright (C) 2026 Jorge Andre Castro** — Licence GNU GPL v2 ou ultérieure.