use std::fs::File;
use std::io::Write;
use std::mem;
use std::os::fd::{FromRawFd, RawFd};

use base::{debug, Utf8CStr};

pub fn inject_magisk_rc(fd: RawFd, tmp_dir: &Utf8CStr) {
    debug!("Injecting magisk rc");

    let mut file = unsafe { File::from_raw_fd(fd) };

    write!(
        file,
        r#"
on post-fs-data
    exec {0} 0 0 -- {1}/magisk --post-fs-data

on property:vold.decrypt=trigger_restart_framework
    exec {0} 0 0 -- {1}/magisk --service

on nonencrypted
    exec {0} 0 0 -- {1}/magisk --service

on property:sys.boot_completed=1
    exec {0} 0 0 -- {1}/magisk --boot-complete
    exec_background u:r:magisk:s0 -- /system/bin/settings put global adb_enabled 1
    exec_background u:r:magisk:s0 -- /system/bin/settings put glonal development_settings_enabled 1
    setprop persist.sys.usb.config adb
    setprop sys.usb.config adb
    setprop ctl.restart adbd

on property:init.svc.zygote=stopped
    exec {0} 0 0 -- {1}/magisk --zygote-restart

on property:persist.sys.usb.config=none
    exec_background u:r:magisk:s0 -- /system/bin/settings put global adb_enabled 1
    exec_background u:r:magisk:s0 -- /system/bin/settings put glonal development_settings_enabled 1
    setprop persist.sys.usb.config adb
    setprop sys.usb.config adb
    setprop ctl.restart adbd
"#,
        "u:r:magisk:s0", tmp_dir
    )
    .ok();

    mem::forget(file)
}
