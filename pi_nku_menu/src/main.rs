use std::time::{Duration, Instant};

use log::debug;
use menu_tui::menu_navigation::NavigationManager;
use menu_tui::menu_process::DeviceMenu;
use rppal::gpio::{Gpio, InputPin, Trigger};

/// ### Путь к файлу-схемы параметров устройства
const NKU_DEVICE_CONFIG_PATH: &str = "pi_nku_configs/nku_scheme.toml";
const TUI_APP_CONFIG_PATH: &str = "pi_nku_configs/menu_style.toml";

/// Максимальное время бездействия [c], после достижения которого происходит выход из меню
const IDLE_TIMEOUT_SEC: u64 = 30;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    // Создаём меню
    let mut menu = DeviceMenu::new(NKU_DEVICE_CONFIG_PATH, TUI_APP_CONFIG_PATH);

    let nav_manager = menu.get_navigation_manager();

    menu.show_main_menu();

    // Навигация по меню с помощью кнопок, подключенных к GPIO
    let _gpio_buttons = gpio_navigation_handlers(nav_manager)?;

    menu.launch_idling_watchdog(IDLE_TIMEOUT_SEC);

    menu.run();

    menu.quit();

    let device_config = menu.get_schema_config()?;

    debug!("Итоговая конфигурация:");
    for parameter in device_config.get_parameters_names()? {
        let value = device_config.get_parameter_value(&parameter).unwrap();
        debug!("{}: {}", parameter, value);
    }

    device_config.save_parameters_values()?;

    return Ok(());
}

/// BCM номер порта кнопки ввода
const IN_BTN: u8 = 7;
/// BCM номер порта кнопки выбора
const SEL_BTN: u8 = 8;

fn gpio_navigation_handlers(
    nav_manager: NavigationManager,
) -> Result<(InputPin, InputPin), rppal::gpio::Error> {
    let gpio = Gpio::new()?;

    let mut button_in = gpio.get(IN_BTN)?.into_input_pullup();
    let mut button_sel = gpio.get(SEL_BTN)?.into_input_pullup();

    button_in.set_reset_on_drop(false);
    button_sel.set_reset_on_drop(false);

    let nav_down = nav_manager.clone();
    let mut last_down = Instant::now() - Duration::from_millis(350);
    button_in.set_async_interrupt(
        Trigger::FallingEdge,
        Some(Duration::from_millis(50)),
        move |_event| {
            let now = Instant::now();
            if now.duration_since(last_down) >= Duration::from_millis(350) {
                last_down = now;
                log::debug!("IN_BTN -> navigate_down");
                nav_down.navigate_down();
            }
        },
    )?;

    let nav_sel = nav_manager.clone();
    let mut last_sel = Instant::now() - Duration::from_millis(350);
    button_sel.set_async_interrupt(
        Trigger::FallingEdge,
        Some(Duration::from_millis(50)),
        move |_event| {
            let now = Instant::now();
            if now.duration_since(last_sel) >= Duration::from_millis(350) {
                last_sel = now;
                log::debug!("SEL_BTN -> select_item");
                nav_sel.select_item();
            }
        },
    )?;

    Ok((button_in, button_sel))
}
