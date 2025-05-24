use std::{collections::HashMap, convert::Infallible, path, str::FromStr, sync::{Arc, LazyLock, Mutex, Weak}};

use super::{file_tag::{get_tag, tag_file}, fs_mon::{FSEvent, FSEventIter}};

type FileTrackersMap = HashMap<String, FileTracker>;
type DirTrackersMap = HashMap<String, Weak<Mutex<DirTracker>>>;

static FILE_TRACKERS: LazyLock<Mutex<FileTrackersMap>> = LazyLock::new(|| -> Mutex<FileTrackersMap> {
    Mutex::new(FileTrackersMap::new())
});

static DIR_TRACKERS: LazyLock<Mutex<DirTrackersMap>> = LazyLock::new(|| -> Mutex<DirTrackersMap> {
    Mutex::new(DirTrackersMap::new())
});

struct DirTracker {
    root: String,
    iter: FSEventIter,
    events: Vec<FSEvent>
}

struct FileTracker {
    dir_tracker: Arc<Mutex<DirTracker>>,
    tracker_state: FileTrackerState
}

#[derive(Debug)]
struct Error;
impl From<Infallible> for Error {
    fn from(_: Infallible) -> Self {
        Error
    }
}

fn get_relative_path(base: &str, path: &str) -> Result<String, Error> {
    let path = std::path::PathBuf::from_str(path)?;
    let base = std::path::PathBuf::from_str(base)?;

    let relative = path.strip_prefix(base);
    match relative {
        Ok(relative) => {
            let relative = relative.to_str().ok_or(Error)?;
            return Ok(relative.into());
        }
        Err(_) => {
            Err(Error)
        }
    }
}

fn check_potential_path(base: &str, partial_path: &str) ->
    Result<(String, String), Error> {
    let full_path: String = 'full: {
        if partial_path.is_empty() {
            break 'full base.into();
        }
        let partial_path = std::path::PathBuf::from_str(partial_path)?;
        let base = std::path::PathBuf::from_str(base)?;

        let full_path = base.join(partial_path);
        let full_path = full_path.to_str().ok_or(Error)?;
        full_path.into()
    };
    let tag = get_tag(&full_path).ok_or(Error)?;
    Ok((tag, full_path.into()))
}

impl FileTracker {
    pub fn update_state(&mut self) {
        let dir_tracker = &*self.dir_tracker.lock().unwrap();
        for event in dir_tracker.events.iter() {
            match event {
                FSEvent::FileRenamedOld(old_path) => {
                    if let FileTrackerState::Certain { id, path } = &self.tracker_state {
                        let res = get_relative_path(old_path, path);
                        if let Ok(relative) = res {
                            self.tracker_state = FileTrackerState::Renaming {
                                id: id.to_owned(),
                                partial_path: relative
                            };
                        }
                    }
                }
                FSEvent::FileRenamedNew(path) => {
                    if let FileTrackerState::Renaming { id, partial_path } = &self.tracker_state {
                        let tag = check_potential_path(path, partial_path);
                        if let Ok((tag, path)) = tag {
                            if id.eq(&tag) {
                                self.tracker_state =
                                    FileTrackerState::Certain { 
                                        id: id.to_owned(),
                                        path
                                    };
                            }
                        }
                    }
                }
                FSEvent::FileRemoved(old_path) => {
                    if let FileTrackerState::Certain { id, path } = &self.tracker_state {
                        let res = get_relative_path(old_path, path);
                        if let Ok(relative) = res {
                            self.tracker_state = FileTrackerState::Moving {
                                id: id.to_owned(),
                                partial_path: relative
                            };
                        }
                    }
                }
                FSEvent::FileAdded(path) => {
                    if let FileTrackerState::Moving { id, partial_path } = &self.tracker_state {
                        let tag = check_potential_path(path, partial_path);
                        if let Ok((tag, path)) = tag {
                            if id.eq(&tag) {
                                self.tracker_state =
                                    FileTrackerState::Certain { 
                                        id: id.to_owned(),
                                        path
                                    };
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub enum FileTrackerState {
    Certain {
        id: String,
        path: String
    },
    Renaming {
        id: String,
        partial_path: String
    },
    Moving {
        id: String,
        partial_path: String
    }
}

fn get_volume_label(file: &str) -> String {
    let mut file = Some(std::path::PathBuf::from_str(file).unwrap()).unwrap();
    while let Some(parent) = file.parent() {
        file = parent.to_path_buf();
    }
    file.to_str().unwrap().into()
}

pub fn unregister_file(file: &str) {
    let trackers = &mut *(*FILE_TRACKERS).lock().unwrap();
    trackers.remove(file);
}

pub fn register_file(file: &str) -> String {
    let trackers = &mut *(*FILE_TRACKERS).lock().unwrap();
    let file_id = tag_file(file);

    let state = FileTrackerState::Certain {
        id: file_id.to_owned(),
        path: file.into()
    };

    let volume = get_volume_label(file);

    let dir_tracker = 'retrieve_tracker: {
        let dir_trackers = &mut *DIR_TRACKERS.lock().unwrap();
        let tracker = dir_trackers.get(&volume);

        if let Some(tracker) = tracker {
            let real_tracker = tracker.upgrade();
            if let Some(tracker) = real_tracker {
                break 'retrieve_tracker tracker;
            }
        }

        let fs_event_iter = FSEventIter::new(&volume).unwrap();
        let dir_tracker = Arc::new(
            Mutex::new(
                DirTracker {
                    root: volume.to_owned(),
                    iter: fs_event_iter,
                    events: vec![]
                }));

        let weak_tracker = Arc::downgrade(&dir_tracker);
        dir_trackers.insert(volume, weak_tracker);

        dir_tracker
    };

    let tracker = FileTracker {
        dir_tracker,
        tracker_state: state
    };

    trackers.insert(file_id.to_owned(), tracker);
    file_id
}

pub fn get_tracker_state(id: &str) -> Option<FileTrackerState> {
    let trackers = &mut *(*FILE_TRACKERS).lock().unwrap();
    let state = trackers.get(id);
    state.map(|x| {
        x.tracker_state.to_owned()
    })
}

pub fn tick() {
    let mut ids = vec![];
    let dir_trackers = &mut *DIR_TRACKERS.lock().unwrap();
    for (id, tracker) in dir_trackers.iter() {
        let tracker = tracker.upgrade();
        if let Some(tracker) = tracker {
            let tracker = &mut *tracker.lock().unwrap();
            tracker.events.clear();
            tracker.iter.tick().unwrap();
            while let Some(event) = tracker.iter.get_event() {
                let event = match event {
                    FSEvent::FileRenamedOld(name) => {
                        let mut full_path = tracker.root.to_owned();
                        full_path.push_str(&name);
                        FSEvent::FileRenamedOld(full_path.into())
                    }
                    FSEvent::FileRenamedNew(name) => {
                        let mut full_path = tracker.root.to_owned();
                        full_path.push_str(&name);
                        FSEvent::FileRenamedNew(full_path.into())
                    }
                    FSEvent::FileAdded(name) => {
                        let mut full_path = tracker.root.to_owned();
                        full_path.push_str(&name);
                        FSEvent::FileAdded(full_path.into())
                    }
                    FSEvent::FileRemoved(name) => {
                        let mut full_path = tracker.root.to_owned();
                        full_path.push_str(&name);
                        FSEvent::FileRemoved(full_path.into())
                    }
                    FSEvent::FileModified(name) => {
                        let mut full_path = tracker.root.to_owned();
                        full_path.push_str(&name);
                        FSEvent::FileModified(full_path.into())
                    }
                };
                tracker.events.push(event);
            }
        }
        else {
            ids.push(id.to_owned());
        }
    }

    for id in ids.iter() {
        dir_trackers.remove(id);
    }

    let file_trackers = &mut *FILE_TRACKERS.lock().unwrap();
    for (_, tracker) in file_trackers.iter_mut() {
        tracker.update_state();
    }
}

