use crate::launchd::message::{
    DISABLE_NAMES, DUMPJPCATEGORY, DUMPSTATE, ENABLE_NAMES, LIST_SERVICES, LOAD_PATHS, PROCINFO,
    UNLOAD_PATHS,
};
use std::convert::TryFrom;
use std::{collections::HashSet, os::unix::prelude::RawFd};

use xpc_sys::{objects::xpc_shmem::XPCShmem, traits::{xpc_pipeable::XPCPipeable, xpc_value::TryXPCValue}, MAP_SHARED, rs_geteuid};

use crate::launchd::entry_status::ENTRY_STATUS_CACHE;
use std::iter::FromIterator;
use xpc_sys::objects::xpc_dictionary::XPCDictionary;
use xpc_sys::objects::xpc_error::XPCError;
use xpc_sys::traits::query_builder::QueryBuilder;

use xpc_sys::enums::{DomainType, SessionType};

pub fn find_in_all<S: Into<String>>(label: S) -> Result<(DomainType, XPCDictionary), XPCError> {
    let label_string = label.into();

    for domain_type in DomainType::System as u64..DomainType::RequestorDomain as u64 {
        let response = XPCDictionary::new()
            .extend(&LIST_SERVICES)
            .entry("type", domain_type)
            .entry("name", label_string.clone())
            .pipe_routine_with_error_handling();

        if response.is_ok() {
            return response.map(|r| (domain_type.into(), r));
        }
    }

    Err(XPCError::NotFound)
}

/// Query for jobs in a domain
pub fn list(domain_type: DomainType, name: Option<String>) -> Result<XPCDictionary, XPCError> {
    XPCDictionary::new()
        .extend(&LIST_SERVICES)
        .with_domain_type_or_default(Some(domain_type))
        .entry_if_present("name", name)
        .pipe_routine_with_error_handling()
}

/// Query for jobs across all domain types
pub fn list_all() -> HashSet<String> {
    let mut everything = vec![
        DomainType::System,
        DomainType::RequestorUserDomain,
        DomainType::RequestorDomain,
    ];

    if rs_geteuid() == 0 {
        everything.push(DomainType::User);
    }

    let list = everything.iter()
    .filter_map(|t| {
        let svc_for_type = list(t.clone(), None)
            .and_then(|d| d.get_as_dictionary(&["services"]))
            .map(|XPCDictionary(ref hm)| hm.keys().map(|k| k.clone()).collect());

        if svc_for_type.is_err() {
            log::error!(
                "[query/list_all]: poll error {}, domain, {}",
                svc_for_type.err().unwrap(),
                t
            );
            None
        } else {
            svc_for_type.ok()
        }
    })
    .flat_map(|k: Vec<String>| k.into_iter());

    HashSet::from_iter(list)
}

pub fn load<S: Into<String>>(
    label: S,
    plist_path: S,
    domain_type: Option<DomainType>,
    session: Option<SessionType>,
    handle: Option<u64>,
) -> Result<XPCDictionary, XPCError> {
    ENTRY_STATUS_CACHE
        .lock()
        .expect("Must invalidate")
        .remove(&label.into());

    XPCDictionary::new()
        .extend(&LOAD_PATHS)
        .with_domain_type_or_default(domain_type)
        .with_session_type_or_default(session)
        .with_handle_or_default(handle)
        .entry("paths", vec![plist_path.into()])
        .pipe_routine_with_error_handling()
}

pub fn unload<S: Into<String>>(
    label: S,
    plist_path: S,
    domain_type: Option<DomainType>,
    session: Option<SessionType>,
    handle: Option<u64>,
) -> Result<XPCDictionary, XPCError> {
    ENTRY_STATUS_CACHE
        .lock()
        .expect("Must invalidate")
        .remove(&label.into());

    XPCDictionary::new()
        .extend(&UNLOAD_PATHS)
        .with_domain_type_or_default(domain_type)
        .with_session_type_or_default(session)
        .with_handle_or_default(handle)
        .entry("paths", vec![plist_path.into()])
        .pipe_routine_with_error_handling()
}

pub fn enable<S: Into<String>>(
    label: S,
    domain_type: DomainType,
) -> Result<XPCDictionary, XPCError> {
    let label_string = label.into();

    XPCDictionary::new()
        .extend(&ENABLE_NAMES)
        .with_domain_type_or_default(Some(domain_type))
        .entry("name", label_string.clone())
        .entry("names", vec![label_string])
        .with_handle_or_default(None)
        .pipe_routine_with_error_handling()
}

pub fn disable<S: Into<String>>(
    label: S,
    domain_type: DomainType,
) -> Result<XPCDictionary, XPCError> {
    let label_string = label.into();

    XPCDictionary::new()
        .extend(&DISABLE_NAMES)
        .with_domain_type_or_default(Some(domain_type))
        .entry("name", label_string.clone())
        .entry("names", vec![label_string])
        .with_handle_or_default(None)
        .pipe_routine_with_error_handling()
}

/// Create a shared shmem region for the XPC routine to write
/// dumpstate contents into, and return the bytes written and
/// shmem region
pub fn dumpstate() -> Result<(usize, XPCShmem), XPCError> {
    let shmem = XPCShmem::new_task_self(
        0x1400000,
        i32::try_from(MAP_SHARED).expect("Must conv flags"),
    )?;

    let response = XPCDictionary::new()
        .extend(&DUMPSTATE)
        .entry("shmem", &shmem.xpc_object)
        .pipe_routine_with_error_handling()?;

    let bytes_written: u64 = response.get(&["bytes-written"])?.xpc_value()?;

    Ok((usize::try_from(bytes_written).unwrap(), shmem))
}

pub fn dumpjpcategory(fd: RawFd) -> Result<XPCDictionary, XPCError> {
    XPCDictionary::new()
        .extend(&DUMPJPCATEGORY)
        .entry("fd", fd)
        .pipe_routine_with_error_handling()
}

pub fn procinfo(pid: i64, fd: RawFd) -> Result<XPCDictionary, XPCError> {
    XPCDictionary::new()
        .extend(&PROCINFO)
        .entry("fd", fd)
        .entry("pid", pid)
        .pipe_routine_with_error_handling()
}
