use std::time::Duration;

pub enum FileSystemEvent {}

unsafe extern "C" {
    fn Boot(dir: *const u8) -> bool;
    fn Tick(dir: *const u8) -> bool;
    fn Peek(dir: *const u8) -> *const FileSystemEvent;
    fn Pop(dir: *const u8);
    fn Shutdown(dir: *const u8);

    fn GetLastErr(len: *mut i32) -> *const u8;
    fn GetAction(event: *const FileSystemEvent) -> i32;
    fn GetFile(event: *const FileSystemEvent, size: *mut i32) -> *const u8;
}

fn get_last_error() -> String {
    unsafe {
        let mut l: i32 = 0;
        let e = GetLastErr(&mut l as *mut i32);
        let e = &*std::ptr::slice_from_raw_parts(e, l as usize);
        let err = String::from_utf8_lossy(e);
        err.to_string()
    }
}

fn to_null_terminated(s: &str) -> Vec<u8> {
    let src_bytes = s.as_bytes();
    let final_size = src_bytes.len() + 1;
    let mut res = Vec::with_capacity(final_size);
    res.resize(final_size, 0);

    let bytes = &mut res[..final_size - 1];
    bytes.copy_from_slice(s.as_bytes());

    res
}

#[derive(Debug)]
pub enum FSEvent {
    FileAdded(String),
    FileRemoved(String),
    FileModified(String),
    FileRenamedOld(String),
    FileRenamedNew(String)
}

pub struct FSEventIter {
    root: String
}

impl FSEventIter {
    pub fn new(root: &str) -> Result<Self, std::io::Error> {
        let dir = to_null_terminated(&root);
        let res = unsafe {
            let dir = dir.as_ptr();
            Boot(dir)
        };

        if !res {
            let error = get_last_error();
            let error = std::io::Error::new(std::io::ErrorKind::Other, error);
            return Err(error);
        }
        let res = FSEventIter {
            root: root.into()
        };
        Ok(res)
    }

    pub fn tick(&self) -> Result<(), std::io::Error> {
        let res = unsafe {
            let dir = to_null_terminated(&self.root);
            Tick(dir.as_ptr())
        };
        if !res {
            let error = get_last_error();
            let error = std::io::Error::new(std::io::ErrorKind::Other, error);
            return Err(error);
        }
        Ok(())
    }

    pub fn get_event(&self) -> Option<FSEvent> {
        unsafe {
            let dir = to_null_terminated(&self.root);
            let evt = Peek(dir.as_ptr());
            if evt.is_null() {
                None
            }
            else {
                let action = GetAction(evt);
                let mut size: i32 = 0;
                let file = GetFile(evt, &mut size as *mut i32);
                let file = &*std::ptr::slice_from_raw_parts(file, size as usize);
                let file = String::from_utf8(file.into()).unwrap();
                
                Pop(dir.as_ptr());

                match action {
                    1 => Some(FSEvent::FileAdded(file.into())),
                    2 => Some(FSEvent::FileRemoved(file.into())),
                    3 => Some(FSEvent::FileModified(file.into())),
                    4 => Some(FSEvent::FileRenamedOld(file.into())),
                    5 => Some(FSEvent::FileRenamedNew(file.into())),
                    _ => None
                }
            }
        }
    }
}

impl Drop for FSEventIter {
    fn drop(&mut self) {
        unsafe {
            let dir = to_null_terminated(&self.root);
            Shutdown(dir.as_ptr());
        }
    }
}

pub fn test() -> std::io::Result<()> {

    let monitor = FSEventIter::new("C:\\")?;
    loop {
        monitor.tick()?;
        while let Some(e) = monitor.get_event() {
            dbg!(e);
        }
    }
    return Ok(());
}
