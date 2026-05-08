# LD2410C

A `no_std` async driver for the LD2410C 24 GHz radar presence sensor.

<img width="804" height="800" alt="image" src="https://github.com/user-attachments/assets/e7c16f93-b781-423d-9471-469d645ae697" />

## Features

- `no_std` — runs on bare-metal microcontrollers with no allocator required
- Async — designed around `async`/`await`; no blocking reads
- HAL-agnostic — bring your own UART via a small trait

## Installation

```toml
[dependencies]
ld2410c = "1.0.0"
```

## Usage

### 1. Implement `UartReader` for your hardware

The crate does not depend on any specific HAL. You connect it to your UART peripheral by implementing the `UartReader` trait:

```rust
use ld2410c::UartReader;

struct MyUart {
    // your HAL's UART type here
}

impl UartReader for MyUart {
    type Error = /* your HAL's error type */;

    async fn read_until_idle(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        // call your HAL's read-until-idle (or equivalent) here
        // return the number of bytes actually written into `buf`
    }
}
```

#### Embassy (STM32) example

```rust
use embassy_stm32::usart::{self, Uart, mode};
use ld2410c::UartReader;

struct Ld2410cUart<'d>(Uart<'d, mode::Async>);

impl UartReader for Ld2410cUart<'_> {
    type Error = usart::Error;

    async fn read_until_idle(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        self.0.read_until_idle(buf).await
    }
}
```

Most async HALs expose a `read_until_idle` method directly (Embassy, RTIC async, etc.), so the implementation is usually a one-liner.

### 2. Create the driver and read frames

```rust
use ld2410c::Ld2410c;

let mut driver = Ld2410c::new(my_uart);
let mut buf = [0u8; 32];

loop {
    match driver.read_frame(&mut buf).await {
        Ok(Some(d)) => {
            info!("Status: {}", d.status);
            info!("Movement target distance: {} cm", d.movement_distance);
            info!("Exercise target energy value: {}", d.movement_energy);
            info!("Distance to stationary target: {} cm", d.stationary_distance);
            info!("Stationary target energy value: {}", d.stationary_energy);
            info!("Detection distance: {} cm", d.detection_distance);
        }
        Ok(None) => warn!("Unknown frame"),
        Err(e) => info!("UART Error: {:?}", e),
    }
}
```

`read_frame` returns `Ok(None)` when the received bytes do not form a valid LD2410C frame (wrong header or insufficient length). In practice this is rare after UART idle detection, but you should handle it gracefully.

## TargetData fields

| Field                 | Type  | Description                                                   |
| --------------------- | ----- | ------------------------------------------------------------- |
| `status`              | `u8`  | Target state (0 = none, 1 = moving, 2 = stationary, 3 = both) |
| `movement_distance`   | `u16` | Distance to moving target in cm                               |
| `movement_energy`     | `u8`  | Reflected energy of moving target (0–100)                     |
| `stationary_distance` | `u16` | Distance to stationary target in cm                           |
| `stationary_energy`   | `u8`  | Reflected energy of stationary target (0–100)                 |
| `detection_distance`  | `u16` | Closest detected target distance in cm                        |

## Frame format

The driver expects the standard LD2410C reporting frame:

```
[F4 F3 F2 F1] [len_lo len_hi] [data_type] [AA] [status]
[mov_lo mov_hi] [mov_energy]
[sta_lo sta_hi] [sta_energy]
[det_lo det_hi]
...
```

Frames shorter than 17 bytes or with a wrong header are silently discarded.
Only target data is parsed (9 bytes between 0xAA HEAD and 0x55 TAIL) and stored into TargetData struct

## License

Licensed under either of [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE) at your option.
