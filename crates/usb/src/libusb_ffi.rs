//! libusb-1.0 FFI bindings
//!
//! All libusb FFI declarations in one place for clean, maintainable code.

use std::ffi::c_void;

// Link to libusb-1.0
#[link(name = "usb-1.0")]
extern "C" {
    // Context management
    pub fn libusb_init(ctx: *mut *mut c_void) -> i32;
    pub fn libusb_exit(ctx: *mut c_void);
    pub fn libusb_set_option(ctx: *mut c_void, option: i32, ...) -> i32;

    // Device enumeration
    pub fn libusb_get_device_list(ctx: *mut c_void, list: *mut *mut *mut c_void) -> isize;
    pub fn libusb_free_device_list(list: *mut *mut c_void, unref_devices: i32);
    pub fn libusb_get_device_descriptor(dev: *mut c_void, desc: *mut DeviceDescriptor) -> i32;
    pub fn libusb_open(dev: *mut c_void, handle: *mut *mut c_void) -> i32;
    pub fn libusb_close(handle: *mut c_void);

    // Device configuration
    pub fn libusb_set_configuration(handle: *mut c_void, configuration: i32) -> i32;
    pub fn libusb_claim_interface(handle: *mut c_void, interface_number: i32) -> i32;
    pub fn libusb_release_interface(handle: *mut c_void, interface_number: i32) -> i32;
    pub fn libusb_kernel_driver_active(handle: *mut c_void, interface_number: i32) -> i32;
    pub fn libusb_detach_kernel_driver(handle: *mut c_void, interface_number: i32) -> i32;

    // Synchronous transfers
    pub fn libusb_control_transfer(
        handle: *mut c_void,
        request_type: u8,
        request: u8,
        value: u16,
        index: u16,
        data: *mut u8,
        length: u16,
        timeout: u32,
    ) -> i32;
    pub fn libusb_bulk_transfer(
        handle: *mut c_void,
        endpoint: u8,
        data: *mut u8,
        length: i32,
        transferred: *mut i32,
        timeout: u32,
    ) -> i32;

    // Asynchronous transfers
    pub fn libusb_alloc_transfer(iso_packets: i32) -> *mut LibusbTransfer;
    pub fn libusb_free_transfer(transfer: *mut LibusbTransfer);
    pub fn libusb_submit_transfer(transfer: *mut LibusbTransfer) -> i32;
    pub fn libusb_cancel_transfer(transfer: *mut LibusbTransfer) -> i32;
    pub fn libusb_handle_events_timeout_completed(
        ctx: *mut c_void,
        tv: *const TimeVal,
        completed: *mut i32,
    ) -> i32;
}

/// USB device descriptor
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct DeviceDescriptor {
    pub b_length: u8,
    pub b_descriptor_type: u8,
    pub bcd_usb: u16,
    pub b_device_class: u8,
    pub b_device_sub_class: u8,
    pub b_device_protocol: u8,
    pub b_max_packet_size0: u8,
    pub id_vendor: u16,
    pub id_product: u16,
    pub bcd_device: u16,
    pub i_manufacturer: u8,
    pub i_product: u8,
    pub i_serial_number: u8,
    pub b_num_configurations: u8,
}

/// libusb transfer structure
#[repr(C)]
pub struct LibusbTransfer {
    pub dev_handle: *mut c_void,
    pub flags: u8,
    pub endpoint: u8,
    pub transfer_type: u8,
    pub timeout: u32,
    pub status: i32,
    pub length: i32,
    pub actual_length: i32,
    pub callback: Option<extern "C" fn(*mut LibusbTransfer)>,
    pub user_data: *mut c_void,
    pub buffer: *mut u8,
    pub num_iso_packets: i32,
}

/// Time value for event handling
#[repr(C)]
pub struct TimeVal {
    pub tv_sec: i64,
    pub tv_usec: i64,
}

// libusb constants
pub const LIBUSB_SUCCESS: i32 = 0;
pub const LIBUSB_ERROR_IO: i32 = -1;
pub const LIBUSB_ERROR_INVALID_PARAM: i32 = -2;
pub const LIBUSB_ERROR_ACCESS: i32 = -3;
pub const LIBUSB_ERROR_NO_DEVICE: i32 = -4;
pub const LIBUSB_ERROR_NOT_FOUND: i32 = -5;
pub const LIBUSB_ERROR_BUSY: i32 = -6;
pub const LIBUSB_ERROR_TIMEOUT: i32 = -7;
pub const LIBUSB_ERROR_OVERFLOW: i32 = -8;
pub const LIBUSB_ERROR_PIPE: i32 = -9;
pub const LIBUSB_ERROR_INTERRUPTED: i32 = -10;
pub const LIBUSB_ERROR_NO_MEM: i32 = -11;
pub const LIBUSB_ERROR_NOT_SUPPORTED: i32 = -12;
pub const LIBUSB_ERROR_OTHER: i32 = -99;

// Transfer types
pub const LIBUSB_TRANSFER_TYPE_CONTROL: u8 = 0;
pub const LIBUSB_TRANSFER_TYPE_ISOCHRONOUS: u8 = 1;
pub const LIBUSB_TRANSFER_TYPE_BULK: u8 = 2;
pub const LIBUSB_TRANSFER_TYPE_INTERRUPT: u8 = 3;

// Transfer status
pub const LIBUSB_TRANSFER_COMPLETED: i32 = 0;
pub const LIBUSB_TRANSFER_ERROR: i32 = 1;
pub const LIBUSB_TRANSFER_TIMED_OUT: i32 = 2;
pub const LIBUSB_TRANSFER_CANCELLED: i32 = 3;
pub const LIBUSB_TRANSFER_STALL: i32 = 4;
pub const LIBUSB_TRANSFER_NO_DEVICE: i32 = 5;
pub const LIBUSB_TRANSFER_OVERFLOW: i32 = 6;

// Request type
pub const LIBUSB_REQUEST_TYPE_VENDOR: u8 = 0x40;
pub const LIBUSB_ENDPOINT_IN: u8 = 0x80;
pub const LIBUSB_ENDPOINT_OUT: u8 = 0x00;

/// Convert libusb error code to string
pub fn error_name(error: i32) -> &'static str {
    match error {
        LIBUSB_SUCCESS => "Success",
        LIBUSB_ERROR_IO => "Input/output error",
        LIBUSB_ERROR_INVALID_PARAM => "Invalid parameter",
        LIBUSB_ERROR_ACCESS => "Access denied",
        LIBUSB_ERROR_NO_DEVICE => "No such device",
        LIBUSB_ERROR_NOT_FOUND => "Entity not found",
        LIBUSB_ERROR_BUSY => "Resource busy",
        LIBUSB_ERROR_TIMEOUT => "Operation timed out",
        LIBUSB_ERROR_OVERFLOW => "Overflow",
        LIBUSB_ERROR_PIPE => "Pipe error",
        LIBUSB_ERROR_INTERRUPTED => "System call interrupted",
        LIBUSB_ERROR_NO_MEM => "Insufficient memory",
        LIBUSB_ERROR_NOT_SUPPORTED => "Operation not supported",
        _ => "Unknown error",
    }
}
