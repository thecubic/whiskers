extern crate libusb;
extern crate clap;

use std::slice;
use std::time::Duration;
use clap::{App, SubCommand};

#[allow(dead_code)]
enum SystemCommand {
	Peek = 0x80,
	Poke = 0x81,
	Ping = 0x82,
	Status = 0x83,
	PokeRegister = 0x84,
	GetClock = 0x85,
	BuildType = 0x86,
	Bootloader = 0x87,
	RFMode = 0x88,
	Compiler = 0x89,
	PartNum = 0x8e,
	Reset = 0x8f,
	ClearCodes = 0x90,
	LedMode = 0x93,
}

#[allow(dead_code)]
enum AppMailbox {
	AppGeneric = 0x01,
	AppDebug = 0xfe,
	AppSystem = 0xff,
}
pub struct RFCatDevice<'a> {
    bus_number: u8,
    address: u8,
    vendor_id: u16,
    product_id: u16,
    handle: libusb::DeviceHandle<'a>,
    descriptor: libusb::DeviceDescriptor,
    language: Option<libusb::Language>,
    timeout: Duration,
    max_input_size: u16,
    in_endpoint_address: u8,
    out_endpoint_address: u8,
}

impl<'a> RFCatDevice<'a> {
    pub fn buildname(&self) -> Result<String, libusb::Error> {
        let mut in_vec = Vec::<u8>::with_capacity(self.max_input_size as usize);
        let in_buf = unsafe { slice::from_raw_parts_mut((&mut in_vec[..]).as_mut_ptr(), in_vec.capacity()) };
        let outvec = vec![AppMailbox::AppSystem as u8,
                            SystemCommand::BuildType as u8,
                            0,
                            0,];
        match self.handle.write_bulk(self.out_endpoint_address, &outvec[..], self.timeout) {
            Ok(_) => (),
            Err(err) => {
                println!("nope: {}", err);
                return Err(err);
            }
        }
        match self.handle.read_bulk(self.in_endpoint_address, in_buf, self.timeout) {
            Ok(rlen) => {
                unsafe { in_vec.set_len(rlen) };
                assert_eq!(in_vec[0], 64);
                assert_eq!(in_vec[1], AppMailbox::AppSystem as u8);
                assert_eq!(in_vec[2], SystemCommand::BuildType as u8);
                let slen = u16::from_le_bytes([in_vec[3], in_vec[4]]);
                let buildname = String::from_utf8(in_vec[5..4+(slen as usize)].to_vec()).unwrap();
                return Ok(buildname);
            },
            Err(err) => {
                println!("nope: {}", err);
                return Err(err);
            }
        }
    }

    pub fn manufacturer(&self) -> Result<String, libusb::Error> {
        match self.handle.read_manufacturer_string(self.language.unwrap(), &self.descriptor, self.timeout) {
            Ok(mstr) => {
                return Ok(mstr);
            },
            Err(err) => {
                return Err(err);
            }
        }
    }

    pub fn product(&self) -> Result<String, libusb::Error> {
        match self.handle.read_product_string(self.language.unwrap(), &self.descriptor, self.timeout) {
            Ok(pstr) => {
                return Ok(pstr);
            },
            Err(err) => {
                return Err(err);
            }
        }
    }
}

fn is_rfcat(usbdd: &libusb::DeviceDescriptor) -> bool {
    match (usbdd.vendor_id(), usbdd.product_id()) {
        // TI
        (0x0451, 0x4715) => true,
        // PandwaRF
        (0x1d50, 0x60ff) => true,
        // RFCat
        (0x1d50, 0x6047) => true,
        (0x1d50, 0x6048) => true,
        // YARD Stick One
        (0x1d50, 0x605b) => true,
        (0x1d50, 0xecc1) => true,
        // nope
        (_, _) => false,
    }
}

#[allow(dead_code)]
fn is_rfcat_bootloader(usbdd: &libusb::DeviceDescriptor) -> bool {
    match (usbdd.vendor_id(), usbdd.product_id()) {
        (0x1d50, 0x6049) => true, 
        (0x1d50, 0x604a) => true,
        (0x1d50, 0x605c) => true,
        (0x1d50, 0xecc0) => true,
        (_, _) => false,
    }
}

fn all_rfcats(context: &libusb::Context) -> Vec<RFCatDevice> {
    let mut rfcat_list: Vec<RFCatDevice> = Vec::new();
    let devices = match context.devices() {
        Ok(k) => k,
        Err(err) => {
            println!("Error: {}", err);
            return rfcat_list;
        },
    };
    for device in devices.iter() {
        let device_desc = match device.device_descriptor() {
            Ok(k) => k,
            Err(err) => {
                println!("Error getting descriptor: {}", err);
                continue
            }
        };
        if is_rfcat(&device_desc) {
            let mut handle = match device.open() {
                Ok(k) => k,
                Err(err) => {
                    println!("Error opening device: {}", err);
                    continue
                }
            };
            let bus_number = device.bus_number();
            let address = device.address();
            let vendor_id = device_desc.vendor_id();
            let product_id = device_desc.product_id();
            let timeout = Duration::from_secs(1);
            let langs = match handle.read_languages(timeout) {
                Ok(k) => k,
                Err(err) => {
                    println!("Error in reading languages: {}", err);
                    continue
                }
            };
            let language: Option<libusb::Language>;
            if langs.len() > 0 {
                language = Some(langs[0]);
            } else {
                language = None;
            }
            let mut in_max_size: u16 = 64;
            let mut in_ep_addr: u8 = 0;
            let mut out_ep_addr: u8 = 0;
            match handle.reset() {
                Ok(k) => k,
                Err(err) => {
                    println!("Error resetting device: {}", err);
                    continue
                }
            };
            for n in 0..device_desc.num_configurations() {
                let config_desc = match device.config_descriptor(n) {
                    Ok(k) => k,
                    Err(_) => continue,
                };
                for interface in config_desc.interfaces() {
                    for interface_desc in interface.descriptors() {
                        for endpoint_desc in interface_desc.endpoint_descriptors() {
                            if endpoint_desc.transfer_type() == libusb::TransferType::Bulk &&
                                 endpoint_desc.direction() == libusb::Direction::In {
                                in_ep_addr = endpoint_desc.address();
                                in_max_size = endpoint_desc.max_packet_size();
                            }
                            if endpoint_desc.transfer_type() == libusb::TransferType::Bulk &&
                                 endpoint_desc.direction() == libusb::Direction::Out {
                                out_ep_addr = endpoint_desc.address();
                            }

                        }
                    }
                }
            }
            match handle.set_active_configuration(1) {
                Ok(k) => k,
                Err(err) => {
                    println!("Error setting configuration: {}", err);
                    continue
                }
            }
            match handle.claim_interface(0) {
                Ok(k) => k,
                Err(err) => {
                    println!("Error claiming interface: {}", err);
                    continue
                }
            }
            match handle.set_alternate_setting(0, 0) {
                Ok(k) => k,
                Err(err) => {
                    println!("Error alternate setting: {}", err);
                    continue
                }
            }
            let exdev = RFCatDevice{
                bus_number: bus_number,
                address: address,
                vendor_id: vendor_id,
                product_id: product_id,
                handle: handle,
                descriptor: device_desc,
                language: language,
                timeout: timeout,
                max_input_size: in_max_size,
                in_endpoint_address: in_ep_addr,
                out_endpoint_address: out_ep_addr,
            };
            rfcat_list.push(exdev);
        }
    }
    rfcat_list
}

fn main() {
    let matches = App::new("whiskers")
        .version("0.1.0")
        .author("Dave Carlson <thecubic@thecubic.net>")
        .about("RFCat driver application")
        .subcommand(
            SubCommand::with_name("buildname")
                .about("display the build name"))
        .subcommand(
            SubCommand::with_name("list")
                .about("list attached RFCats"))
        .get_matches();
    match matches.subcommand_name() {
        Some("buildname") => {
            let context = libusb::Context::new().unwrap();
            let rfcats = all_rfcats(&context);
            for rfcat in rfcats.iter() {
                println!("RFCat: b{:03} d{:03}d v{:04x} p{:04x}",
                         rfcat.bus_number,
                         rfcat.address,
                         rfcat.vendor_id,
                         rfcat.product_id);
                match rfcat.buildname() {
                    Ok(buildname) => println!("  buildname: {}", buildname),
                    Err(err) => println!("  Error: {}", err),
                }
            }
        },
        Some("list") => {
            let context = libusb::Context::new().unwrap();
            let rfcats = all_rfcats(&context);
            for rfcat in rfcats.iter() {
                println!("RFCat: b{:03} d{:03} v{:04x} p{:04x}",
                         rfcat.bus_number,
                         rfcat.address,
                         rfcat.vendor_id,
                         rfcat.product_id);
                match rfcat.manufacturer() {
                    Ok(mstr) => {
                        println!("  {}", mstr);
                    },
                    Err(err) => {
                        println!("  Error: {}", err);
                    }
                }
                match rfcat.product() {
                    Ok(pstr) => {
                        println!("  {}", pstr);
                    },
                    Err(err) => {
                        println!("  Error: {}", err);
                    }
                }
            }
        }
        None | _ => (),
    }
}


