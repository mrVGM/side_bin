use std::{collections::HashMap, sync::{Arc, LazyLock, Mutex}};

use super::file_tag::tag_file;

type FileTrackersMap = HashMap<String, FileTracker>;

static FILE_TRACKERS: LazyLock<Mutex<FileTrackersMap>> = LazyLock::new(|| -> Mutex<FileTrackersMap> {
    Mutex::new(FileTrackersMap::new())
});

struct DirTracker;
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

pub fn register_file(file: &str) -> String {
    let trackers = &mut *(*FILE_TRACKERS).lock().unwrap();
    let file_id = tag_file(file);

    let state = FileTrackerState::Certain {
        id: file_id.to_owned(),
        path: file.into()
    };

    let tracker = FileTracker {
        dir_tracker: DirTracker.into(),
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


