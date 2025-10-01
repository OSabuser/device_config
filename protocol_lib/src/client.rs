use log::{debug, error, warn};

use crate::mu_frame::MUFrame;
use std::time::Duration;

/// Количество попыток установить соединение c MCU
const HANDSHAKE_ATTEMPTS: u8 = 5;

pub struct HostClient {
    serial_port: Box<dyn serialport::SerialPort + 'static>,
}

impl HostClient {
    /// Создание соединения host - интерфейсная плата over UART
    pub fn connect(
        port_name: &str,
        baudrate: u32,
        timeout: Duration,
    ) -> Result<HostClient, String> {
        let serial_port = serialport::new(port_name, baudrate)
            .timeout(timeout)
            .open()
            .unwrap_or_else(|_| panic!("Unable to open serial port: {port_name}"));

        Self::try_handshake(serial_port)
    }

    /// Попытка установить соединение с устройством
    fn try_handshake(instance: Box<dyn serialport::SerialPort + 'static>) -> Result<Self, String> {
        let mut attempts: u8 = 1;

        let mut client_connection = HostClient {
            serial_port: instance,
        };

        // Цикл попыток установить соединение
        'handshake_loop: loop {
            warn!("Handshake attempt: {attempts}");

            let answer = client_connection.send_request("hello");

            if let Ok(response) = answer {
                warn!("Response from device: {response}");

                if response.as_bytes() == b"Hi!\r\n" {
                    return Ok(client_connection);
                }
            } else {
                error!("Handshake: {}", answer.clone().unwrap_err());
            }

            attempts += 1;

            if attempts > HANDSHAKE_ATTEMPTS {
                break 'handshake_loop;
            }
        }
        Err("Handshake failed!".to_string())
    }

    /// Отправка запроса на устройство, возврат полученного отклика
    pub fn send_request(&mut self, request: &str) -> Result<String, String> {
        let mut frame = MUFrame::new();
        frame
            .set_data(format!("{}{}", request, "\n").as_bytes().to_vec())
            .map_err(|e| e.to_string())?;
        crate::send_proto_message(frame, &mut self.serial_port)?;

        let new_frame =
            crate::recv_proto_message(&mut self.serial_port).map_err(|e| e.to_string())?;

        Ok(String::from_utf8(new_frame.get_data().to_vec()).map_err(|e| e.to_string())?)
    }
}
