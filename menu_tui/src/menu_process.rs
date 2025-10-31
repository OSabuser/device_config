//! # Модуль отображения меню
//!
//! ## Навигация
//! > Для навигации по меню используется клавиши стрелка вверх/вниз
//! Выбор элемента в меню происходит клавишей Enter
//! `В случае отсутствия данных клавиш в системе, можно эмулировать их с помощью модуля menu_navigation
//!
//! ## Параметры меню
//! > Параметры меню подтягиваются из файла-схемы TOML с помощью крейта config_lib

use std::time::{Duration, Instant};

use config_lib::device_config::DeviceConfig;
use cursive::{
    Cursive, CursiveExt,
    event::{Event, EventResult, EventTrigger},
};

use crate::{
    menu_navigation::NavigationManager, user_parameters::DeviceParameters,
    views::main_view::show_main_view,
};

pub struct DeviceMenu {
    siv: Cursive,
    nav_manager: NavigationManager,
    scheme_config: DeviceConfig,
}

pub(crate) struct MenuAppState {
    pub navigation_manager: NavigationManager,
    pub inner_config: DeviceParameters,
}

impl DeviceMenu {
    pub fn new(path_to_scheme: &str, theme_path: &str) -> Self {
        // Получение конфигурации устройства
        let device_config = DeviceConfig::create_parameter_list(path_to_scheme)
            .expect("Ошибка загрузки config_scheme");

        let mut siv = Cursive::default();

        siv.set_global_callback('q', |siv: &mut Cursive| {
            siv.quit();
        });
        // TODO: обработка ошибок
        let mut device_parameters: DeviceParameters = DeviceParameters::new();

        // Заполнение списка пользовательских параметров данными из `DeviceConfig`
        device_parameters
            .load_user_config(&device_config)
            .expect("Ошибка загрузки пользовательских параметров!");

        let nav_manager = NavigationManager::new(siv.cb_sink().clone());

        // TODO: Cursive logs example
        if let Err(_) = siv.load_theme_file(theme_path) {
            println!("{}", format!("Не удалось загрузить тему: {theme_path}"));
        }

        // Сохраняем и NavigationManager и конфигурацию
        let app_state = MenuAppState {
            navigation_manager: nav_manager.clone(),
            inner_config: device_parameters,
        };

        // При работе в меню с исподьзованием обычной клавиатуры, обновляем last_activity (активность пользователя)
        siv.set_on_pre_event_inner(EventTrigger::from_fn(|e| matches!(e, Event::Key(_))), {
            let nav_manager_clone = nav_manager.clone();
            move |_event: &Event| -> Option<EventResult> {
                nav_manager_clone.set_last_activity(Instant::now());
                None
            }
        });

        siv.set_user_data(app_state);

        DeviceMenu {
            siv,
            nav_manager,
            scheme_config: device_config,
        }
    }

    /// Показать главное меню
    pub fn show_main_menu(&mut self) {
        self.nav_manager.set_current_view("main_menu");
        show_main_view(&mut self.siv);
    }

    /// Получить менеджер навигации для управления извне
    pub fn get_navigation_manager(&self) -> NavigationManager {
        self.nav_manager.clone()
    }

    /// Запустить главный цикл обработки событий
    pub fn run(&mut self) {
        self.siv.run();
    }

    /// ## Запуск таймера, который проверяет активность пользователя
    /// > Если пользователь не активен в течение `timeout_seconds`, то приложение закрывается.
    /// > Каждый раз, когда пользователь нажимает одну из кнопок навигации, сбрасывается таймер.
    pub fn launch_idling_watchdog(&mut self, timeout_seconds: u64) {
        let nav_manager = self.get_navigation_manager();
        let cb_sink = self.siv.cb_sink().clone();

        std::thread::spawn(move || {
            loop {
                std::thread::sleep(Duration::from_secs(1));

                // Проверяем активность
                let is_active = {
                    let last = nav_manager.get_last_activity();
                    last.elapsed() < Duration::from_secs(timeout_seconds)
                };

                if !is_active {
                    cb_sink
                        .send(Box::new(|s: &mut Cursive| {
                            s.quit();
                        }))
                        .ok();
                    break;
                }
            }
        });
    }

    /// Закрытие приложения
    pub fn quit(&mut self) {
        self.siv.quit();
    }

    /// Получить текущую конфигурацию
    pub fn get_schema_config(&mut self) -> Result<DeviceConfig, String> {
        // TODO: обработка ошибок
        if let Some(app_state) = self.siv.take_user_data::<MenuAppState>() {
            app_state
                .inner_config
                .update_user_config(&mut self.scheme_config)?;
        }

        Ok(self.scheme_config.clone())
    }
}
