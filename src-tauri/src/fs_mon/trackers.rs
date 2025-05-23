use std::{collections::HashMap, str::FromStr, sync::{Arc, LazyLock, Mutex, Weak}};

use super::file_tag::tag_file;

type FileTrackersMap = HashMap<String, FileTracker>;
type DirTrackersMap = HashMap<String, Weak<DirTracker>>;

static FILE_TRACKERS: LazyLock<Mutex<FileTrackersMap>> = LazyLock::new(|| -> Mutex<FileTrackersMap> {
    Mutex::new(FileTrackersMap::new())
});

static DIR_TRACKERS: LazyLock<Mutex<DirTrackersMap>> = LazyLock::new(|| -> Mutex<DirTrackersMap> {
    Mutex::new(DirTrackersMap::new())
});

struct DirTracker(String);
struct FileTracker {
    dir_tracker: Arc<DirTracker>,
    tracker_state: FileTrackerState
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

        let dir_tracker = Arc::new(DirTracker(volume.to_owned()));
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


