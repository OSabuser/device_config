use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;

use crate::logger::RollingFileLogger;
use config_lib::serial_config::SerialPortConfig;
use log::{error, info, warn};
use protocol_lib::inc_parser::{FrameParser, ParseResult};
use signal_hook::consts::{SIGINT, SIGTERM};
use signal_hook::iterator::Signals;
use std::time::Duration;

/// Путь до файла-схемы параметров Serial порта устройства
const SERIAL_PORT_CONFIG_PATH: &str = "rk_nku_configs/rk3399_scheme.toml";
/// Максимальное время ожидания ответа от устройства (`Heartbeat устройства - 60 секунд`)
const BOARD_RESPONSE_TIMEOUT_MS: std::time::Duration = std::time::Duration::from_secs(65);
/// Размер буфера для хранения принятых данных перед записью в файл
const BUFFER_SIZE: usize = 20;
/// Путь директории для хранения логов
const LOG_DIR: &str = "logs";
/// Интервал между записями в лог файл
const FLUSH_INTERVAL_MS: u64 = 100;

mod logger;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    // Инициализация обработчиков сигналов ОС
    let mut signals = Signals::new(&[SIGINT, SIGTERM])?;
    let handle = signals.handle();
    let running = Arc::new(AtomicBool::new(true));
    let running_clone = Arc::clone(&running);

    // Создание потока для обработки сигналов
    let signal_thread = thread::spawn(move || {
        for sig in signals.forever() {
            match sig {
                SIGINT => {
                    warn!("\n[*] Получен сигнал SIGINT.");
                    running_clone.store(false, Ordering::SeqCst);
                    break;
                }
                SIGTERM => {
                    warn!("\n[*] Получен сигнал SIGTERM.");
                    running_clone.store(false, Ordering::SeqCst);
                    break;
                }
                _ => {}
            }
        }
    });

    // Настройка последовательного порта
    let serial_port_config = SerialPortConfig::new(SERIAL_PORT_CONFIG_PATH)?;
    let port_name = serial_port_config.get_serial_name();
    let baudrate = serial_port_config.get_serial_baudrate();

    info!("{}: {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
    info!("Порт: {:<30}", port_name);
    info!("Скорость: {:<24}", baudrate);
    info!("Место хранения логов: {:<27}", LOG_DIR);

    let mut serial_port = serialport::new(port_name, baudrate)
        .timeout(BOARD_RESPONSE_TIMEOUT_MS)
        .open()?;

    let mut logger = RollingFileLogger::new(LOG_DIR, BUFFER_SIZE)?;
    let mut parser = FrameParser::new();

    let mut buffer = [0u8; 512];
    let mut last_flush = std::time::Instant::now();
    let mut frame_counter = 0;

    while running.load(Ordering::SeqCst) {
        match serial_port.read(&mut buffer) {
            Ok(n) if n > 0 => {
                // Обработка принятого сообщения побайтно
                for &byte in &buffer[0..n] {
                    match parser.process_raw_byte(byte) {
                        ParseResult::FrameReady(frame) => {
                            logger.log_frame(frame.get_opcode(), frame.get_data().as_slice())?;

                            frame_counter += 1;

                            if frame_counter % 10 == 0 {
                                info!("[*] Записано {} сообщений", frame_counter);
                            }
                        }
                        ParseResult::Error(e) => {
                            logger.log_error(&e)?;
                            error!("[!] Ошибка парсинга: {}", e);
                        }
                        ParseResult::Incomplete => {}
                    }
                }
                last_flush = std::time::Instant::now();
            }
            Ok(_) => {}
            Err(e) => {
                let error_msg = format!("Ошибка при чтении порта: {}", e);
                logger.log_error(&error_msg)?;
                error!("[!] {}", error_msg);

                // Try to reconnect
                thread::sleep(Duration::from_millis(500));
            }
        }

        // Periodic flush
        if last_flush.elapsed() > Duration::from_millis(FLUSH_INTERVAL_MS) {
            logger.flush()?;
        }
    }

    // Graceful shutdown
    warn!("\n[*] Ожидания закрытие потока чтения сигналов ОС...");
    handle.close();
    let _ = signal_thread.join();

    // Final flush
    logger.flush()?;

    let stats = logger.stats();

    info!("########################################################");
    info!(".           Статистика сессии           .");
    info!("Всего сообщений: {:<26}в•‘", stats.total_messages);
    info!("Всего ошибок: {:<26}в•‘", stats.total_errors);
    info!("Создано файлов с логами: {:<25}в•‘", stats.files_rotated);
    info!("Текущий файл: {:<30}в•‘", logger.current_log_file());
    info!("########################################################");

    Ok(())
}
