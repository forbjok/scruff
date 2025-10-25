pub trait UiHandler {
    fn begin_load(&mut self, filename: &str);
    fn end_load(&mut self);

    fn begin_clean(&mut self, path: &str);
    fn end_clean(&mut self);

    fn delete_dir(&mut self, path: &str);
    fn delete_file(&mut self, path: &str);
}
