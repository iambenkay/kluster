use core::ptr::{addr_of, addr_of_mut, read_volatile, write_volatile};

const MBOX_RESPONSE: u32 = 0x8000_0000;
const MBOX_FULL: u32 = 0x8000_0000;
const MBOX_EMPTY: u32 = 0x4000_0000;

struct MboxAddress;

impl MboxAddress {
    #[inline(always)]
    fn root() -> usize {
        #[cfg(feature = "rpi5")]
        return 0x10_7C01_3880;

        #[cfg(feature = "rpi4")]
        0xFE00_B880
    }
    #[inline(always)]
    fn read() -> *const u32 {
        (Self::root() + 0x00) as *const u32
    }
    #[inline(always)]
    fn status() -> *const u32 {
        (Self::root() + 0x18) as *const u32
    }
    #[inline(always)]
    fn write() -> *mut u32 {
        (Self::root() + 0x20) as *mut u32
    }
}

#[repr(C, align(16))]
struct MailboxBuffer([u32; 36]);

impl MailboxBuffer {
    const fn new() -> MailboxBuffer {
        MailboxBuffer([0; 36])
    }
}

static mut MBOX_BUFFER: MailboxBuffer = MailboxBuffer::new();

/// set value at mailbox index
pub fn mbox_set(i: usize, v: u32) {
    unsafe {
        let base = addr_of_mut!(MBOX_BUFFER.0) as *mut u32;
        write_volatile(base.add(i), v);
    }
}

/// get value at mailbox index
pub fn mbox_get(i: usize) -> u32 {
    unsafe {
        let base = addr_of!(MBOX_BUFFER.0) as *const u32;
        read_volatile(base.add(i))
    }
}

/// Send mailbox request on channel `channel` and wait for response
pub fn mbox_call(channel: u32) -> bool {
    let addr = addr_of!(MBOX_BUFFER) as usize as u32;
    let channel = (addr & !0xF) | (channel & 0xF);

    while is_mbox_full() {
        core::hint::spin_loop();
    }

    write_mbox_request(channel);

    loop {
        while is_mbox_empty() {
            core::hint::spin_loop();
        }
        if does_mbox_response_match(channel) {
            return mbox_get(1) == MBOX_RESPONSE;
        }
    }
}

fn is_mbox_full() -> bool {
    unsafe { (read_volatile(MboxAddress::status()) & MBOX_FULL) != 0 }
}

fn is_mbox_empty() -> bool {
    unsafe { (read_volatile(MboxAddress::status()) & MBOX_EMPTY) != 0 }
}

fn write_mbox_request(request: u32) {
    unsafe { write_volatile(MboxAddress::write(), request) }
}

fn does_mbox_response_match(request: u32) -> bool {
    unsafe { read_volatile(MboxAddress::read()) == request }
}
