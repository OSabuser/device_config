pub mod client;
mod mu_frame;

use crate::mu_frame::MUFrame;

use log::trace;
use std::{
    io::{Read, Write},
    thread,
};

/// Задержка перед приемом отклика от интерфейсной платы
const ANSWER_DELAY_MS: u64 = 500;

/// Отправка сообщения
fn send_proto_message<Writer: Write>(data: MUFrame, mut writer: Writer) -> Result<(), String> {
    let bytes = data.serialize();
    writer.write_all(&bytes).map_err(|e| e.to_string())?;

    // Время для составления ответа
    thread::sleep(std::time::Duration::from_millis(ANSWER_DELAY_MS));

    Ok(())
}

/// Прием сообщения
fn recv_proto_message<Reader: Read>(mut reader: Reader) -> Result<MUFrame, String> {
    let mut raw_frame = Vec::new();
    let mut read_buffer = [0; 256];

    // Чтение отклика от интерфейсной платы
    reader.read(&mut read_buffer).map_err(|e| e.to_string())?;

    trace!("Received raw message: {read_buffer:?}");

    // Сборка сообщения
    let prefix = read_buffer[0];
    raw_frame.push(prefix);

    let payload_length = read_buffer[1];
    raw_frame.push(payload_length);

    let opcode = read_buffer[2];
    raw_frame.push(opcode);

    // 3[prefix, payload_length, opcode] + payload_length
    let end_of_data_idx = 3 + payload_length as usize;

    let payload = &read_buffer[3..end_of_data_idx];
    raw_frame.extend_from_slice(payload);

    let crc_low = read_buffer[end_of_data_idx];
    raw_frame.push(crc_low);
    let crc_high = read_buffer[1 + end_of_data_idx];
    raw_frame.push(crc_high);

    let postfix = read_buffer[2 + end_of_data_idx];
    raw_frame.push(postfix);

    let frame = MUFrame::deserialize(&raw_frame).map_err(|e| e.to_string())?;

    Ok(frame)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_send_and_recv() {
        let mut frame_to_send = MUFrame::new();
        frame_to_send
            .set_data(b"get server_info\n".to_vec())
            .unwrap();

        let mut buf = Vec::new();

        send_proto_message(frame_to_send.clone(), &mut buf).unwrap();

        let received_frame = recv_proto_message(&buf[..]).unwrap();
        assert_eq!(received_frame.get_data(), frame_to_send.get_data());
        assert_eq!(received_frame, frame_to_send);
    }
}
