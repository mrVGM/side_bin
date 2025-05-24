use std::{sync::LazyLock, time::SystemTime, str::FromStr};

use mac_address::get_mac_address;
use uuid::{ClockSequence, Timestamp, Uuid};

struct Clock(u128);

impl Clock {
    fn new() -> Self {
        let now = SystemTime::now();
        let duration_since_epoch = now.duration_since(SystemTime::UNIX_EPOCH).unwrap();
        let time_stamp = duration_since_epoch.as_nanos();

        Clock(time_stamp)
    }
}

impl ClockSequence for Clock {
    type Output = u128;

    fn generate_sequence(&self, seconds: u64, subsec_nanos: u32) -> Self::Output {
        self.0 + (1_000_000_000 * seconds as u128) + (subsec_nanos as u128)
    }
}

static CLOCK: LazyLock<Clock> = LazyLock::new(|| -> Clock {
    Clock::new()
});

pub fn tag_file(file: &str) -> String {
    let mac_address = get_mac_address().unwrap().unwrap();
    let clock = &*CLOCK;
    
    let uuid = Uuid::new_v1(Timestamp::now(clock), &mac_address.bytes());
    let uuid = format!("{}", uuid);

    let file = file.to_owned() + ":dd_tag";
    let file = std::path::PathBuf::from_str(&file).unwrap();
    std::fs::write(file, &uuid).unwrap();

    uuid
}

pub fn get_tag(file: &str) -> Option<String> {
    let file = file.to_owned() + ":dd_tag";
    let res = std::fs::read(file);
    match res {
        Ok(res) => {
            let uuid = String::from_utf8(res).unwrap();
            Some(uuid)
        }
        Err(_) => {
            None
        }
    }
}

