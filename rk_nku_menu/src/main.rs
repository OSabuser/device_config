use config_lib::device_config::DeviceConfig;
use enigo::{Direction::Click, Enigo, Key, Keyboard, Settings};
use log::{debug, error};
use menu_tui::menu_rendering::{MainMenuStates, show_main_dialog};
use std::sync::atomic::AtomicBool;
use std::time::Duration;
use sysfs_gpio::{Direction, Edge, Pin};

/// ### Путь к файлу-схемы параметров устройства
const NKU_DEVICE_CONFIG_PATH: &str = "rk_nku_configs/nku_scheme.toml";

/// Расчёт номера пина в порядке SYSFS
/// ```
/// bank = 0; // GPIO0_B5 => 0, bank ∈ [0,4]
/// group = 1; // GPIO0_B5 => 1, group ∈ {(A=0), (B=1), (C=2), (D=3)}
/// X = 5; // GPIO0_B5 => 5, X ∈ [0,7]
/// number = group * 8 + X = 1 * 8 + 5 = 13;
/// pin = bank * 32 + number = 0 * 32 + 13 = 13;
/// ```

/// SYSFS номер порта кнопки ввода
const S1_BTN: u8 = 124; //124
/// SYSFS номер порта кнопки выбора
const S2_BTN: u8 = 125; //125
/// Максимальное время бездействия [c], после достижения которого происходит выход из меню
const IDLE_TIMEOUT_SEC: u32 = 30;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let mut device_config = DeviceConfig::create_parameter_list(NKU_DEVICE_CONFIG_PATH)?;

    static BTN_PRESSED: AtomicBool = AtomicBool::new(false);
    static STOP_FLAG: AtomicBool = AtomicBool::new(false);

    // Открытие потоков для обслуживания EXT прерываний
    let down_irq_thread = std::thread::spawn(move || {
        let input = Pin::new(S2_BTN as u64);
        debug!("down_irq_thread launched!");
        input.with_exported(|| {
            input.set_direction(Direction::In)?;
            input.set_edge(Edge::RisingEdge)?;
            let mut poller = input.get_poller()?;

            while !STOP_FLAG.load(std::sync::atomic::Ordering::SeqCst) {
                if let Some(pin_value) = poller.poll(1000)? {
                    debug!("{}", format!("Pin value: {}", pin_value));
                    if pin_value == 1 {
                        let mut enigo = Enigo::new(&Settings::default()).unwrap();
                        enigo.key(Key::DownArrow, Click).unwrap();
                        BTN_PRESSED.store(true, std::sync::atomic::Ordering::SeqCst);
                        std::thread::sleep(Duration::from_millis(500));
                        debug!("DownArrow pressed");
                    }
                }
            }
            Ok(())
        })
    });

    let return_irq_thread = std::thread::spawn(move || {
        let input = Pin::new(S1_BTN as u64);
        debug!("return_irq_thread launched!");
        input.with_exported(|| {
            input.set_direction(Direction::In)?;
            input.set_edge(Edge::RisingEdge)?;
            let mut poller = input.get_poller()?;

            while !STOP_FLAG.load(std::sync::atomic::Ordering::SeqCst) {
                if let Some(pin_value) = poller.poll(1000)? {
                    debug!("{}", format!("Pin value: {}", pin_value));
                    if pin_value == 1 {
                        let mut enigo = Enigo::new(&Settings::default()).unwrap();
                        enigo.key(Key::Return, Click).unwrap();
                        BTN_PRESSED.store(true, std::sync::atomic::Ordering::SeqCst);
                        std::thread::sleep(Duration::from_millis(500));
                        debug!("Return pressed");
                    }
                }
            }

            Ok(())
        })
    });

    // Поток TUI меню
    std::thread::spawn(move || {
        loop {
            match show_main_dialog(&mut device_config) {
                Ok(MainMenuStates::ConfigurationState) => {
                    debug!("Configuration state");
                    continue;
                }
                Ok(MainMenuStates::ExitState) => {
                    debug!("The user has chosen to exit...");
                    break;
                }
                Err(e) => {
                    error!("Menu error: {}", e);
                    break;
                }
            }
        }

        STOP_FLAG.store(true, std::sync::atomic::Ordering::SeqCst);
        debug!("The user has chosen to exit...");
    });

    // Главный поток - проверка бездействия
    let mut idle_counter: u32 = 0;
    while !STOP_FLAG.load(std::sync::atomic::Ordering::SeqCst) {
        if BTN_PRESSED.load(std::sync::atomic::Ordering::SeqCst) {
            idle_counter = 0;
            BTN_PRESSED.store(false, std::sync::atomic::Ordering::SeqCst);
        } else {
            idle_counter += 1;
            if idle_counter > IDLE_TIMEOUT_SEC * 10 {
                STOP_FLAG.store(true, std::sync::atomic::Ordering::SeqCst);
                debug!("Idle timeout has been reached, exiting...");
            }
        }
        std::thread::sleep(Duration::from_millis(100));
    }

    if down_irq_thread.join().is_err() {
        debug!("Unable to stop down_irq_thread");
    }

    if return_irq_thread.join().is_err() {
        debug!("Unable to stop return_irq_thread");
    }

    return Ok(());
}
