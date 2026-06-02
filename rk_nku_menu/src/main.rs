use std::time::{Duration, Instant};

use log::debug;
use menu_tui::menu_navigation::NavigationManager;
use menu_tui::menu_process::DeviceMenu;
use sysfs_gpio::{Direction, Edge, Pin};
/// ### Путь к файлу-схемы параметров устройства
const NKU_DEVICE_CONFIG_PATH: &str = "rk_nku_configs/nku_scheme.toml";
const TUI_APP_CONFIG_PATH: &str = "rk_nku_configs/menu_style.toml";

/// Максимальное время бездействия [c], после достижения которого происходит выход из меню
const IDLE_TIMEOUT_SEC: u64 = 30;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    // Создаём меню
    let mut menu = DeviceMenu::new(NKU_DEVICE_CONFIG_PATH, TUI_APP_CONFIG_PATH);

    let nav_manager = menu.get_navigation_manager();

    menu.show_main_menu();

    // Навигация по меню с помощью кнопок, подключенных к GPIO
    gpio_navigation_handlers(nav_manager);

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

fn gpio_navigation_handlers(nav_manager: NavigationManager) {
    // Кнопка 1 (GPIO): Навигация вниз
    let nav_down = nav_manager.clone();
    std::thread::spawn(move || {
        let input = Pin::new(S1_BTN as u64);
        debug!("return_irq_thread launched!");
        input.with_exported(|| {
            input.set_direction(Direction::In)?;
            input.set_edge(Edge::RisingEdge)?;
            let mut poller = input.get_poller()?;

            let mut last_press = Instant::now() - Duration::from_millis(1000);
            let debounce = Duration::from_millis(350);

            loop {
                if let Some(pin_value) = poller.poll(1000)? {
                    if pin_value == 1 {
                        let now = Instant::now();
                        if now.duration_since(last_press) >= debounce {
                            last_press = now;
                            nav_down.navigate_down();
                        } else {
                            // Событие проигнорировано как дребезг
                        }
                    }
                }
            }
        })
    });

    // Кнопка 2 (GPIO): Выбор/подтверждение
    let nav_down = nav_manager.clone();
    std::thread::spawn(move || {
        let input = Pin::new(S2_BTN as u64);
        debug!("return_irq_thread launched!");
        input.with_exported(|| {
            input.set_direction(Direction::In)?;
            input.set_edge(Edge::RisingEdge)?;
            let mut poller = input.get_poller()?;

            let mut last_press = Instant::now() - Duration::from_millis(1000);
            let debounce = Duration::from_millis(350);
            loop {
                if let Some(pin_value) = poller.poll(1000)? {
                    if pin_value == 1 {
                        let now = Instant::now();
                        if now.duration_since(last_press) >= debounce {
                            last_press = now;
                            nav_down.select_item();
                        } else {
                            // Событие проигнорировано как дребезг
                        }
                    }
                }
            }
        })
    });
}
