use crate::{
    MUFrame,
    mu_frame::{SYNC1, SYNC2},
};

/// Состояния парсера фреймов
#[derive(Debug, Clone, Copy, PartialEq)]
enum ParserState {
    WaitingForSync1,
    WaitingForLength,
    WaitingForOpcode,
    WaitingForPayload { length: u8, read: u8 },
    WaitingForCrcLow,
    WaitingForCrcHigh,
    WaitingForSync2,
}

/// Контекст парсера фрейма
pub struct FrameParser {
    state: ParserState,
    frame_buffer: Vec<u8>,
}

/// Результат парсинга данных принятых по UART
pub enum ParseResult {
    /// Фрейм полностью принят
    FrameReady(MUFrame),
    /// Ошибка парсинга
    Error(String),
    /// Фрейм принят не полностью
    Incomplete,
}

impl FrameParser {
    pub fn new() -> Self {
        Self {
            state: ParserState::WaitingForSync1,
            frame_buffer: Vec::with_capacity(256),
        }
    }

    pub fn process_raw_byte(&mut self, byte: u8) -> ParseResult {
        match self.state {
            // Чтение SYNC1 байта
            ParserState::WaitingForSync1 => {
                if byte == SYNC1 {
                    self.frame_buffer.clear();
                    self.frame_buffer.push(byte);
                    self.state = ParserState::WaitingForLength;
                }
                ParseResult::Incomplete
            }
            // Чтение длины payload
            ParserState::WaitingForLength => {
                if byte == 0 {
                    self.state = ParserState::WaitingForSync1;
                    return ParseResult::Error("Invalid payload length".to_string());
                }
                self.frame_buffer.push(byte);
                self.state = ParserState::WaitingForOpcode;
                ParseResult::Incomplete
            }
            // Чтение opcode
            ParserState::WaitingForOpcode => {
                if !MUFrame::is_opcode_correct(byte) {
                    self.state = ParserState::WaitingForSync1;
                    return ParseResult::Error("Invalid opcode".to_string());
                }
                self.frame_buffer.push(byte);
                let length = self.frame_buffer[1];
                self.state = ParserState::WaitingForPayload { length, read: 0 };
                ParseResult::Incomplete
            }
            // Чтение payload
            ParserState::WaitingForPayload { length, read } => {
                self.frame_buffer.push(byte);
                let new_read = read + 1;

                self.state = if new_read >= length {
                    ParserState::WaitingForCrcLow
                } else {
                    ParserState::WaitingForPayload {
                        length,
                        read: new_read,
                    }
                };
                ParseResult::Incomplete
            }
            // Чтение CRC - LSB
            ParserState::WaitingForCrcLow => {
                self.frame_buffer.push(byte);
                self.state = ParserState::WaitingForCrcHigh;
                ParseResult::Incomplete
            }
            // Чтение CRC - MSB
            ParserState::WaitingForCrcHigh => {
                self.frame_buffer.push(byte);
                self.state = ParserState::WaitingForSync2;
                ParseResult::Incomplete
            }
            // Чтение SYNC2
            ParserState::WaitingForSync2 => {
                self.frame_buffer.push(byte);

                if byte != SYNC2 {
                    self.state = ParserState::WaitingForSync1;
                    return ParseResult::Error("Invalid SYNC2 byte".to_string());
                }

                // Попытка десериализации в MuFrame
                let result = match MUFrame::deserialize(&self.frame_buffer) {
                    Ok(frame) => ParseResult::FrameReady(frame),
                    Err(e) => ParseResult::Error(e),
                };
                self.state = ParserState::WaitingForSync1;
                result
            }
        }
    }

    /// Сброс состояния парсера в начальное
    pub fn reset(&mut self) {
        self.state = ParserState::WaitingForSync1;
        self.frame_buffer.clear();
    }
}

impl Default for FrameParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::MUFrame;

    #[test]
    fn test_parsing() {
        let serialized_vec = vec![
            0xAA, 0x1A, 0xC0, 0x23, 0x53, 0x54, 0x4D, 0x3A, 0x4C, 0x30, 0x3A, 0x52, 0x31, 0x36,
            0x3A, 0x41, 0x31, 0x3A, 0x53, 0x30, 0x3A, 0x4D, 0x30, 0x3A, 0x45, 0x23, 0x0D, 0x0A,
            0x00, 0x80, 0x77, 0xBB,
        ];

        let mut parser = FrameParser::new();
        let deserilized_frame = MUFrame::deserialize(&serialized_vec).unwrap();
        assert!(deserilized_frame.validate_frame().is_ok());

        for byte in serialized_vec {
            let result = parser.process_raw_byte(byte);
            if let ParseResult::FrameReady(frame) = result {
                assert_eq!(frame, deserilized_frame);
            }
        }
    }
}
