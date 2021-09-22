use anyhow::Result;
use caps::{CapSet, Capability};

pub fn run() -> Result<()> {
    let caps_to_drop = [
        Capability::CAP_AUDIT_CONTROL,
        Capability::CAP_AUDIT_READ,
        Capability::CAP_AUDIT_WRITE,
        Capability::CAP_BLOCK_SUSPEND,
        Capability::CAP_DAC_READ_SEARCH,
        Capability::CAP_FSETID,
        Capability::CAP_IPC_LOCK,
        Capability::CAP_MAC_ADMIN,
        Capability::CAP_MAC_OVERRIDE,
        Capability::CAP_MKNOD,
        Capability::CAP_SETFCAP,
        Capability::CAP_SYSLOG,
        Capability::CAP_SYS_ADMIN,
        Capability::CAP_SYS_BOOT,
        Capability::CAP_SYS_MODULE,
        Capability::CAP_SYS_NICE,
        Capability::CAP_SYS_RAWIO,
        Capability::CAP_SYS_RESOURCE,
        Capability::CAP_SYS_TIME,
        Capability::CAP_WAKE_ALARM,
    ];

    for cap in &caps_to_drop {
        caps::drop(None, CapSet::Bounding, *cap)?;
        caps::drop(None, CapSet::Inheritable, *cap)?;
    }

    Ok(())
}
