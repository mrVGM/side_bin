use std::{collections::HashMap, str::FromStr, sync::{Arc, LazyLock, Mutex, Weak}};

use super::{file_tag::tag_file, fs_mon::{FSEvent, FSEventIter}};

type FileTrackersMap = HashMap<String, FileTracker>;
type DirTrackersMap = HashMap<String, Weak<Mutex<DirTracker>>>;

static FILE_TRACKERS: LazyLock<Mutex<FileTrackersMap>> = LazyLock::new(|| -> Mutex<FileTrackersMap> {
    Mutex::new(FileTrackersMap::new())
});

static DIR_TRACKERS: LazyLock<Mutex<DirTrackersMap>> = LazyLock::new(|| -> Mutex<DirTrackersMap> {
    Mutex::new(DirTrackersMap::new())
});

struct DirTracker {
    iter: FSEventIter,
    events: Vec<FSEvent>
}

struct FileTracker {
    dir_tracker: Arc<Mutex<DirTracker>>,
    tracker_state: FileTrackerState
}

impl FileTracker {
    pub fn update_state(&mut self) {
        println!("file tracker state update");
        let dir_tracker = &*self.dir_tracker.lock().unwrap();
        for event in dir_tracker.events.iter() {
            dbg!(event);
        }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub enum FileTrackerState {
    Certain {
        id: String,
        path: String
    },
    Uncertain {
        id: String
    },
    Lost {
        id: String
    }
}

fn get_volume_label(file: &str) -> String {
    let mut file = Some(std::path::PathBuf::from_str(file).unwrap()).unwrap();
    while let Some(parent) = file.parent() {
        file = parent.to_path_buf();
    }
    file.to_str().unwrap().into()
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
                iter: fs_event_iter,
                events: vec![]
            })
        );
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

