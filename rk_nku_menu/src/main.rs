use gpiocdev::Request;
use log::debug;
use menu_tui::menu_navigation::NavigationManager;
use menu_tui::menu_process::DeviceMenu;
use std::time::Duration;
/// ### Путь к файлу-схемы параметров устройства
const NKU_DEVICE_CONFIG_PATH: &str = "rk_smart_configs/smart_scheme.toml";
const TUI_APP_CONFIG_PATH: &str = "rk_smart_configs/menu_style.toml";

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

/// ligpiod номер порта кнопки ввода
const S1_BTN: (&str, u8) = ("/dev/gpiochip3", 16);
/// ligpiod номер порта кнопки выбора
const S2_BTN: (&str, u8) = ("/dev/gpiochip2", 1);
/// ## Симуляция обработчиков GPIO кнопок
/// ### Расчёт номера пина в порядке libgpiod
/// ```
/// bank = 0; // GPIO0_B5 => 0, bank ∈ [0,4]
/// group = 1; // GPIO0_B5 => 1, group ∈ {(A=0), (B=1), (C=2), (D=3)}
/// X = 5; // GPIO0_B5 => 5, X ∈ [0,7]
/// number = group * 8 + X = 1 * 8 + 5 = 13;
/// pin = bank * 32 + number = 0 * 32 + 13 = 13;
///
/// gpiochip_number = pin_number / 32; // Номер банка
/// line_offset = pin_number % 32; // Номер пина в банке
/// ```
///
fn gpio_navigation_handlers(nav_manager: NavigationManager) {
    // Кнопка 1 (GPIO): Навигация вниз
    let nav_down = nav_manager.clone();
    std::thread::spawn(move || {
        let down_btn_req = Request::builder()
            .on_chip(S1_BTN.0)
            .with_line(S1_BTN.1 as u32)
            .as_input()
            .with_edge_detection(gpiocdev::line::EdgeDetection::RisingEdge)
            .request()
            .unwrap();
        debug!("down_irq_thread launched!");

        loop {
            if down_btn_req.has_edge_event().unwrap() {
                if let Ok(edge) = down_btn_req.read_edge_event() {
                    if edge.kind == gpiocdev::line::EdgeKind::Rising {
                        debug!("DownArrow pressed");
                        // Симуляция нажатия кнопки DOWN
                        nav_down.navigate_down();
                        std::thread::sleep(Duration::from_millis(400));
                    }
                }
            }
        }
    });

    // Кнопка 2 (GPIO): Выбор/подтверждение
    let nav_select = nav_manager.clone();
    std::thread::spawn(move || {
        let return_btn_req = Request::builder()
            .on_chip(S2_BTN.0)
            .with_line(S2_BTN.1 as u32)
            .as_input()
            .with_edge_detection(gpiocdev::line::EdgeDetection::RisingEdge)
            .request()
            .unwrap();
        debug!("return_irq_thread launched!");
        loop {
            if return_btn_req.has_edge_event().unwrap() {
                if let Ok(edge) = return_btn_req.read_edge_event() {
                    if edge.kind == gpiocdev::line::EdgeKind::Rising {
                        debug!("Return pressed");
                        // Симуляция нажатия кнопки ENTER
                        nav_select.select_item();
                        std::thread::sleep(Duration::from_millis(400));
                    }
                }
            }
        }
    });
}
