# py32-bind-hal

[![Crates.io](https://img.shields.io/crates/v/py32-bind-hal.svg)](https://crates.io/crates/py32-bind-hal)

This project aims to provide a more complete HAL (Hardware Abstraction Layer).

The project uses the vendor-provided C SDK and operates peripherals through bindings, then wraps these C APIs for easy use in Rust.

Users can also directly use FFI to perform complex operations without manipulating registers.

## Supported MCU:

### ---PY32F0xx Series---

**Puya** PY32F002A, PY32F003, PY32F030

**Xinlinggo** XL32F003*, XL32F002A*

**Luat** AIR001

| Peripherals/Functions | Bindings | Easy-to-use func   | embedded-hal |
| --------------------- | -------- | ------------------ | ------------ |
| GPIO                  | ✔        | ✔                  | ✔            |
| RCC                   | ✔        | ✔                  | N/C          |
| Power                 | ✔        | ✔(only sleep/stop) | N/C          |
| DMA                   | ✔        | ✔                  | N/C          |
| RTC                   | ✔        |                    | N/C          |
| WDG                   | ✔        |                    | N/C          |
| PWM/TIMER             | ✔        | ✔(only PWM now)    | N/C          |

| Peripherals/Functions | Bindings | Easy-to-use func | embedded-hal/io | embedded-hal/io-async | Polling | DMA | IT  |
| --------------------- | -------- | ---------------- | --------------- | --------------------- | ------- | --- | --- |
| EXTI                  | ✔        | ✔                | ✔               | ✔                     | N/C     | N/C | ✔   |
| I2C                   | ✔        | ✔                | ✔               |                       | ✔       |     |     |
| ADC                   | ✔        | ✔                | N/C             | N/C                   | ✔       | ✔   |     |
| UART                  | ✔        | ✔                |                 |                       | ✔       |     |     |
| SPI                   | ✔        |                  |                 |                       |         |     |     |

N/C: mcu hardware or embedded-hal not support

✖: no plan or hard to impl

WIP: work in progress

Others:

| Interrupt(cortex-m-rt) | Embassy Time-Driver | HAL-Ticks |
| ---------------------- | ------------------- | --------- |

## Why use bindings?

Taking STM32 as an example, there are many excellent HALs available: [embassy](https://github.com/embassy-rs/embassy)   [stm32-rs](https://github.com/stm32-rs)

This crate’s performance, ROM, and RAM usage are far inferior to these HALs. 

However, most Rust HALs are maintained by the community or enthusiasts and do not receive vendor support. Especially for microcontrollers with fewer users, there are not enough people to maintain the HAL, or in the end, only basic functions can be used.

This crate requires little maintenance and does not require dealing with registers. Even if there are unwrapped functions, others can easily supplement or directly call FFI.

## py32csdk-hal-sys

The CSDK and bindings for py32 are maintained here: [py32csdk-hal-sys](https://github.com/decaday/py32csdk-hal-sys), and this package already includes precompiled static library file and `bindings.rs` for quick use. However, if you want to recompile and generate bindings, it will be troublesome, please check its Docs. You need to enable the `recompile` feature.

## Old Verisons

You can see old versions at [bind-hal  -  crates.io](https://crates.io/crates/bind-hal) and [decaday/bind-hal (github.com)](https://github.com/decaday/bind-hal) .