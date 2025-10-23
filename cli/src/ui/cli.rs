use scruff::ui::UiHandler;

#[derive(Default)]
pub struct CliUiHandler;

impl UiHandler for CliUiHandler {
    fn begin_load(&mut self, _filename: &str) {}
    fn end_load(&mut self) {}

    fn begin_clean(&mut self, _path: &str) {}
    fn end_clean(&mut self) {}

    fn delete_file(&mut self, path: &str) {
        eprintln!("D {path}");
    }
}
