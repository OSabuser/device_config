//! # Модуль навигации по меню
//!
//! > Позволяет имитировать нажатия определенных клавиш в процессе работы с меню.
//! Используется если в системе отсутствует клавиатура.
//!
//!

use std::sync::{Arc, Mutex};
use std::time::Instant;

use cursive::event::{Event, EventResult, Key};
use cursive::views::{CircularFocus, ListView, SelectView};
use cursive::{CbSink, Cursive, View};

// TODO: Перенести в отдельный модуль
enum ActiveMenuView {
    MainView,
    ConfigView,
    ExitView,
    UpdateView,
}

impl From<&str> for ActiveMenuView {
    fn from(value: &str) -> Self {
        match value {
            "main_menu" => ActiveMenuView::MainView,
            "config_menu" => ActiveMenuView::ConfigView,
            "exit_menu" => ActiveMenuView::ExitView,
            "update_menu" => ActiveMenuView::UpdateView,
            _ => ActiveMenuView::MainView,
        }
    }
}

/// Менеджер навигации для управления меню c вызывающей стороны
pub struct NavigationManager {
    // CbSink  — это thread-safe канал, позволяющий отправлять callback-функции из любого потока.
    // Cursive автоматически обрабатывает эти callback’и в своем event loop, обеспечивая потокобезопасность
    cb_sink: CbSink,
    current_view: Arc<Mutex<String>>,
    last_activity: Arc<Mutex<Instant>>,
}

impl NavigationManager {
    /// Создать новый менеджер навигации
    pub fn new(cb_sink: CbSink) -> Self {
        NavigationManager {
            cb_sink,
            current_view: Arc::new(Mutex::new("main_menu".to_string())),
            last_activity: Arc::new(Mutex::new(Instant::now())),
        }
    }

    /// Получить время последнего нажатия клавиши
    pub fn get_last_activity(&self) -> Instant {
        self.last_activity.lock().unwrap().clone()
    }

    /// Установить время последнего нажатия клавиши
    pub fn set_last_activity(&self, instant: Instant) {
        let mut last_time = self.last_activity.lock().unwrap();
        *last_time = instant;
    }

    /// Установить текущий активный View (Cursive)
    pub(crate) fn set_current_view(&self, view_name: &str) {
        let mut current = self.current_view.lock().unwrap();
        *current = view_name.to_string();
    }

    /// Получить имя текущего View (Cursive)
    pub(crate) fn get_current_view(&self) -> String {
        self.current_view.lock().unwrap().clone()
    }

    /// Имитация нажатия клавиши вниз
    pub fn navigate_down(&self) {
        self.set_last_activity(Instant::now());
        let view_name = self.get_current_view();
        self.send_event(&view_name, Event::Key(Key::Down));
    }

    /// Имитация нажатия клавиши вверх
    pub fn navigate_up(&self) {
        self.set_last_activity(Instant::now());
        let view_name = self.get_current_view();
        self.send_event(&view_name, Event::Key(Key::Up));
    }

    /// Имитация нажатия клавиши Enter
    pub fn select_item(&self) {
        let mut last_time = self.last_activity.lock().unwrap();
        *last_time = Instant::now();
        let view_name = self.get_current_view();
        self.send_event(&view_name, Event::Key(Key::Enter));
    }

    /// ## Отправка события `event` текущему рантайму `Cursive` для активного `View`
    /// Отправка по каналу с помощью `cb_sink: Sender`
    fn send_event(&self, view_name: &str, event: Event) {
        let view_name = view_name.to_string();
        let event_clone = event.clone();

        self.cb_sink
            .send(Box::new(move |s: &mut Cursive| {
                Self::dispatch_event_globally(s, &view_name, event_clone);
            }))
            .ok(); // Используем .ok() чтобы не паниковать при закрытии
    }

    // Отправка события `event` в разные типы view
    fn dispatch_event_globally(siv: &mut cursive::Cursive, view_name: &str, event: Event) {
        // Сначала пытаемся отправить событие в активный слой (верхний)
        // Это позволит обработать popup окна
        let result = siv.screen_mut().on_event(event.clone());

        match result {
            EventResult::Consumed(maybe_callback) => {
                // Событие обработано, выполняем callback если есть
                if let Some(callback) = maybe_callback {
                    callback(siv);
                }
                return;
            }
            EventResult::Ignored => {

                // Событие проигнорировано верхним слоем
                // Пытаемся отправить в именованный view
            }
        }

        // Fallback: отправка в конкретный именованный view
        // Для CircularFocus<SelectView<i32>> - главное меню
        if let Some(result) = siv
            .call_on_name(view_name, |view: &mut CircularFocus<SelectView<i32>>| {
                view.on_event(event.clone())
            })
        {
            if let EventResult::Consumed(Some(callback)) = result {
                callback(siv);
                return;
            }
        }

        // Для CircularFocus<ListView> - меню параметров устройства
        if let Some(result) = siv.call_on_name(view_name, |view: &mut CircularFocus<ListView>| {
            view.on_event(event.clone())
        }) {
            if let EventResult::Consumed(Some(callback)) = result {
                callback(siv);
            }
        }
    }
}

/// Реализация клонирования для использования менеджера в разных потоках
impl Clone for NavigationManager {
    fn clone(&self) -> Self {
        NavigationManager {
            cb_sink: self.cb_sink.clone(),
            current_view: Arc::clone(&self.current_view),
            last_activity: Arc::clone(&self.last_activity),
        }
    }
}
