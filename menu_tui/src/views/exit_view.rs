use cursive::Cursive;

/// Закрытие приложения с сохранением параметров
pub(crate) fn show_exit_view(siv: &mut Cursive) {
    siv.quit();
}
