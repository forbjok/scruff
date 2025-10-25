use std::{collections::VecDeque, fs, io, path::PathBuf};

pub struct DirTraverser {
    path: PathBuf,
    ignore_globset: Option<globset::GlobSet>,
}

pub enum DirTraverserEvent {
    EnterDir(PathBuf),
    ExitDir(PathBuf),
    File(PathBuf),
}

#[derive(Default)]
pub struct DirTraverserIterator {
    ignore_globset: globset::GlobSet,
    process_queue: Vec<DirTraverserEvent>,
    entry_queue: VecDeque<Result<fs::DirEntry, io::Error>>,
    output_queue: VecDeque<DirTraverserEvent>,
    cur_dir_process_index: usize,
}

impl DirTraverser {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            ignore_globset: None,
        }
    }

    pub fn ignore(mut self, ignore_globset: globset::GlobSet) -> Self {
        self.ignore_globset = Some(ignore_globset);

        self
    }
}

impl IntoIterator for DirTraverser {
    type Item = anyhow::Result<DirTraverserEvent>;

    type IntoIter = DirTraverserIterator;

    fn into_iter(self) -> Self::IntoIter {
        let ignore_globset = self.ignore_globset.unwrap_or_default();
        let process_queue = vec![DirTraverserEvent::EnterDir(self.path)];

        DirTraverserIterator {
            ignore_globset,
            process_queue,
            ..Default::default()
        }
    }
}

impl Iterator for DirTraverserIterator {
    type Item = anyhow::Result<DirTraverserEvent>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.next_internal() {
            Ok(Some(v)) => Some(Ok(v)),
            Ok(None) => None,
            Err(err) => Some(Err(err)),
        }
    }
}

impl DirTraverserIterator {
    fn next_internal(&mut self) -> anyhow::Result<Option<DirTraverserEvent>> {
        loop {
            if let Some(entry) = self.output_queue.pop_front() {
                return Ok(Some(entry));
            }

            while let Some(entry) = self.entry_queue.pop_front() {
                let entry = entry?;
                let path = entry.path();

                if self.ignore_globset.is_match(&path) {
                    continue;
                }

                if entry.file_type()?.is_dir() {
                    self.process_queue
                        .insert(self.cur_dir_process_index, DirTraverserEvent::EnterDir(path));

                    continue;
                }

                return Ok(Some(DirTraverserEvent::File(entry.file_name().into())));
            }

            let Some(event) = self.process_queue.pop() else {
                return Ok(None);
            };

            match event {
                DirTraverserEvent::EnterDir(path) => {
                    self.output_queue.push_front(DirTraverserEvent::EnterDir(path.clone()));

                    self.process_queue.push(DirTraverserEvent::ExitDir(path.clone()));
                    self.cur_dir_process_index = self.process_queue.len();

                    self.entry_queue.extend(fs::read_dir(&path)?);
                }
                event => self.output_queue.push_front(event),
            }
        }
    }
}
