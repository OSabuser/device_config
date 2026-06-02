/// ### Модуль логгирования в файл
/// - Предоставляет функционал для записи логов в файл с временными метками
/// - Каждые сутки создает новый файл для записи
/// - Предоставляет статистику:  количество корректных сообщений, количество ошибок, количество созданных файлов
use chrono::{Local, NaiveDate};
use log::{trace, warn};
use protocol_lib::mu_frame::MUFrame;
use std::fs::{File, OpenOptions};
use std::io::{self, BufWriter, Write};
pub struct RollingFileLogger {
    /// Директория для хранения логов
    log_dir: String,
    /// Текущее время с начала старта
    current_date: NaiveDate,
    /// Текущий файл, в который ведётся запись
    current_file: Option<BufWriter<File>>,
    /// Буфер для промежуточного хранения логов
    buffer: Vec<String>,
    /// Емкость буфера (хранит логи перед записью в файл)
    max_buffer_size: usize,
    /// Статистика логгера
    stats: LoggerStats,
}

#[derive(Debug, Clone, Default)]
/// Статистика текущей сессии логгирования
pub struct LoggerStats {
    /// Корректно принятые, распакованные сообщения
    pub total_messages: u64,
    /// Количество возникших ошибок (парсинг, таймаут приема)
    pub total_errors: u64,
    /// Количество созданных файлов
    pub files_rotated: u64,
}

impl RollingFileLogger {
    pub fn new(log_dir: &str, max_buffer_size: usize) -> io::Result<Self> {
        std::fs::create_dir_all(log_dir)?;

        let logger = Self {
            log_dir: log_dir.to_string(),
            current_date: Local::now().date_naive(),
            current_file: None,
            buffer: Vec::new(),
            max_buffer_size,
            stats: LoggerStats::default(),
        };

        Ok(logger)
    }

    /// Логгирование распакованного сообщения
    pub fn log_frame(&mut self, opcode: u8, payload: &[u8]) -> io::Result<()> {
        let payload_str = String::from_utf8_lossy(payload);
        let message = format!(
            "СООБЩЕНИЕ | ТИП: {} | ДАННЫЕ: {}",
            MUFrame::get_opcode_description(opcode),
            payload_str
        );

        trace!("{}", message);

        self.log(message)?;
        self.stats.total_messages += 1;

        Ok(())
    }

    /// Логгирование ошибки
    pub fn log_error(&mut self, error: &str) -> io::Result<()> {
        let message = format!("ОШИБКА | {}", error);
        self.log(message)?;
        self.stats.total_errors += 1;

        Ok(())
    }

    fn log(&mut self, message: String) -> io::Result<()> {
        let now = Local::now();
        let new_date = now.date_naive();

        // Прошли сутки - создаем новый файл для записи
        if new_date != self.current_date {
            self.flush()?;
            self.current_file = None;
            self.current_date = new_date;
            self.stats.files_rotated += 1;

            warn!("[LOG] Date changed, new log file created: {}", new_date);
        }
        if self.current_file.is_none() {
            let file = self.open_log_file()?;
            self.current_file = Some(BufWriter::new(file));
        }

        let timestamped = format!("[{}] {}", now.format("%Y-%m-%d %H:%M:%S%.3f"), message);

        self.buffer.push(timestamped);

        // Запись в файл после заполнения буфера
        if self.buffer.len() >= self.max_buffer_size {
            self.flush()?;
        }

        Ok(())
    }

    /// **Открытие файла для записи**
    /// > Файл будет создан, если он не существует
    fn open_log_file(&self) -> io::Result<File> {
        let filename = format!(
            "{}/uart_log_{}.txt",
            self.log_dir,
            self.current_date.format("%Y-%m-%d")
        );

        OpenOptions::new().create(true).append(true).open(&filename)
    }

    pub fn flush(&mut self) -> io::Result<()> {
        if let Some(ref mut file) = self.current_file {
            for line in self.buffer.drain(..) {
                writeln!(file, "{}", line)?;
            }
            file.flush()?;
        }
        Ok(())
    }

    /// Получение статистики текущей сессии логгирования
    pub fn stats(&self) -> &LoggerStats {
        &self.stats
    }

    /// Получение имени файла, в который в данный момент ведётся запись
    pub fn current_log_file(&self) -> String {
        format!(
            "{}/uart_log_{}.txt",
            self.log_dir,
            self.current_date.format("%Y-%m-%d")
        )
    }
}

impl Drop for RollingFileLogger {
    fn drop(&mut self) {
        let _ = self.flush();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_log_error() {
        let mut logger = RollingFileLogger::new("tests_output", 10).unwrap();
        logger.log_error("Test error").unwrap();
        assert_eq!(logger.stats.total_errors, 1);
    }

    #[test]
    fn test_log_frame() {
        let mut logger = RollingFileLogger::new("tests_output", 10).unwrap();
        logger
            .log_frame(0x01, "That was test frame #1".as_bytes())
            .unwrap();
        assert_eq!(logger.stats.total_messages, 1);
    }

    #[test]
    fn test_flush() {
        let mut logger = RollingFileLogger::new("tests_output", 10).unwrap();
        logger
            .log_frame(0x01, "That was test frame #2".as_bytes())
            .unwrap();
        logger.flush().unwrap();
        assert_eq!(logger.stats.total_messages, 1);
    }
}
