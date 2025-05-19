//! Implementation for UEFI using EFI_RNG_PROTOCOL
use crate::Backend;
use crate::Error;
use core::{
    mem::MaybeUninit,
    ptr::{self, null_mut, NonNull},
    sync::atomic::{AtomicPtr, Ordering::Relaxed},
};
use r_efi::{
    efi::{BootServices, Handle},
    protocols::rng,
};

extern crate std;

#[cfg(not(target_os = "uefi"))]
compile_error!("`efi_rng` backend can be enabled only for UEFI targets!");

static RNG_PROTOCOL: AtomicPtr<rng::Protocol> = AtomicPtr::new(null_mut());

#[cold]
#[inline(never)]
fn init() -> Result<NonNull<rng::Protocol>, Error> {
    const HANDLE_SIZE: usize = size_of::<Handle>();

    let boot_services = std::os::uefi::env::boot_services()
        .ok_or(Error::new_custom(BOOT_SERVICES_UNAVAILABLE))?
        .cast::<BootServices>();

    let mut handles = [ptr::null_mut(); 16];
    // `locate_handle` operates with length in bytes
    let mut buf_size = handles.len() * HANDLE_SIZE;
    let mut guid = rng::PROTOCOL_GUID;
    let ret = unsafe {
        ((*boot_services.as_ptr()).locate_handle)(
            r_efi::efi::BY_PROTOCOL,
            &mut guid,
            null_mut(),
            &mut buf_size,
            handles.as_mut_ptr(),
        )
    };

    if ret.is_error() {
        return Err(Error::from_uefi_code(ret.as_usize()));
    }

    let handles_len = buf_size / HANDLE_SIZE;
    let handles = handles.get(..handles_len).ok_or(Error::UNEXPECTED)?;

    let system_handle = std::os::uefi::env::image_handle();
    for &handle in handles {
        let mut protocol: MaybeUninit<*mut rng::Protocol> = MaybeUninit::uninit();

        let mut protocol_guid = rng::PROTOCOL_GUID;
        let ret = unsafe {
            ((*boot_services.as_ptr()).open_protocol)(
                handle,
                &mut protocol_guid,
                protocol.as_mut_ptr().cast(),
                system_handle.as_ptr(),
                ptr::null_mut(),
                r_efi::system::OPEN_PROTOCOL_GET_PROTOCOL,
            )
        };

        let protocol = if ret.is_error() {
            continue;
        } else {
            let protocol = unsafe { protocol.assume_init() };
            NonNull::new(protocol).ok_or(Error::UNEXPECTED)?
        };

        // Try to use the acquired protocol handle
        let mut buf = [0u8; 8];
        let mut alg_guid = rng::ALGORITHM_RAW;
        let ret = unsafe {
            ((*protocol.as_ptr()).get_rng)(
                protocol.as_ptr(),
                &mut alg_guid,
                buf.len(),
                buf.as_mut_ptr(),
            )
        };

        if ret.is_error() {
            continue;
        }

        RNG_PROTOCOL.store(protocol.as_ptr(), Relaxed);
        return Ok(protocol);
    }
    Err(Error::new_custom(NO_RNG_HANDLE))
}

pub struct UefiBackend;

unsafe impl Backend for UefiBackend {
    #[inline]
    unsafe fn fill_ptr(dest: *mut u8, len: usize) -> Result<(), Error> {
        let protocol = match NonNull::new(RNG_PROTOCOL.load(Relaxed)) {
            Some(p) => p,
            None => init()?,
        };

        let mut alg_guid = rng::ALGORITHM_RAW;
        let ret =
            unsafe { ((*protocol.as_ptr()).get_rng)(protocol.as_ptr(), &mut alg_guid, len, dest) };

        if ret.is_error() {
            Err(Error::from_uefi_code(ret.as_usize()))
        } else {
            Ok(())
        }
    }

    #[inline]
    fn describe_custom_error(n: u16) -> Option<&'static str> {
        match n {
            BOOT_SERVICES_UNAVAILABLE => None, // TODO: Custom error message?
            NO_RNG_HANDLE => None,             // TODO: Custom error message?
            _ => None,
        }
    }
}

const BOOT_SERVICES_UNAVAILABLE: u16 = 10;
const NO_RNG_HANDLE: u16 = 11;
