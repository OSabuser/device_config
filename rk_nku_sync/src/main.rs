mod nku_client;
mod serial_config;

use std::str::FromStr;

use clap::Parser;
use log::{error, warn};

/// Количество попыток выполнить запрос
const REQUEST_ATTEMPTS: u8 = 5;

#[derive(Parser)]
#[command(author = "Akimov Dmitry MU LLC", name = "nku_sync", version = "0.1.0", about, long_about = None)]
struct Args {
    /// Тип команды: pull - запрос сохраненных в устройстве настроек, push - отправка новых настроек
    #[arg(short = 'm', long = "mode")]
    mode: CommandMode,
}

fn main() -> Result<(), String> {
    let args = Args::parse();

    env_logger::init();
    warn!("rk_nku_sync> command mode: {:?}", args.mode);

    let mut nku_client = nku_client::NkuClient::new()?;

    match args.mode {
        CommandMode::Pull => pull_parameters(&mut nku_client)?,
        CommandMode::Push => push_parameters(&mut nku_client)?,
    }

    Ok(())
}

/// Отправка настроек на устройство
fn push_parameters(client: &mut nku_client::NkuClient) -> Result<(), String> {
    // Цикл попыток установить соединение
    let mut attempts: u8 = 1;
    'push_request_loop: loop {
        warn!("Push request attempt: {attempts}");
        if client.push_parameters_to_device().is_ok() {
            break 'push_request_loop;
        }
        let result = client.push_parameters_to_device();

        if result.is_ok() {
            break 'push_request_loop;
        }

        error!(
            "{}",
            format!("Failed to push parameters: {result:?}").as_str()
        );

        attempts += 1;

        if attempts > REQUEST_ATTEMPTS {
            return Err("Push request failed!".to_string());
        }
    }

    // Запуск стриминга данных от станции
    attempts = 1;
    'start_streaming_loop: loop {
        warn!("Start streaming attempt: {attempts}");

        let result = client.start_elevator_data_streaming(nku_client::StreamingMode::OnChangeMode);

        if result.is_ok() {
            break 'start_streaming_loop;
        }
        error!(
            "{}",
            format!("Failed to push parameters: {result:?}").as_str()
        );
        attempts += 1;

        if attempts > REQUEST_ATTEMPTS {
            return Err("Start streaming failed!".to_string());
        }
    }

    Ok(())
}

/// Получение настроек, сохраненных в устройстве
fn pull_parameters(client: &mut nku_client::NkuClient) -> Result<(), String> {
    let mut attempts: u8 = 1;
    // Цикл попыток установить соединение
    'pull_request_loop: loop {
        warn!("Pull request attempt: {attempts}");
   
        let result = client.pull_parameters_from_device();
        if result.is_ok() {
            break 'pull_request_loop;
        }
        error!(
            "{}",
            format!("Failed to push parameters: {result:?}").as_str()
        );

        attempts += 1;

        if attempts > REQUEST_ATTEMPTS {
            return Err("Pull request failed!".to_string());
        }
    }

    Ok(())
}

#[derive(Clone, Debug)]
enum CommandMode {
    Pull,
    Push,
}

impl FromStr for CommandMode {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "pull" => Ok(CommandMode::Pull),
            "push" => Ok(CommandMode::Push),
            _ => Err(format!("Unknown command mode: {s}")),
        }
    }
}
