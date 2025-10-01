# Настройка рабочего окружения

## Кросскомпиляция

- [cargo-zigbuild](https://github.com/rust-cross/cargo-zigbuild)
- [cross-rs](https://github.com/cross-rs/cross)

### Сборка

> Важно: необходимо использовать версию линковщика, совпадающую с версией, установленной на целевом устройстве

- cross-rs: `cross build --target=arm-unknown-linux-gnueabihf --bin <crate> --release`

- zigbuild: `cargo zigbuild --target=arm-unknown-linux-gnueabihf.[версия линкера на целевом устройстве] --bin <crate> --release`

> Для кросскомпиляции `serialport` (libudev не поддерживается на MacOS & WIN64) указать фича-флаг `default-features = false`

## Загрузка на целевое устройство

`scp -P [port_number] [path/to/bin] [hostname]@[ip_address]:/home/[user-name]`  
