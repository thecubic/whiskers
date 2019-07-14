extern crate libusb;
extern crate clap;

// use std::slice;
use std::time::Duration;
use clap::{App, Arg, SubCommand};
use std::fs::File;
use std::path::Path;
use std::error::Error;
use std::io::Write;


pub struct RFCatBLDevice<'a> {
    bus_number: u8,
    address: u8,
    vendor_id: u16,
    product_id: u16,
    handle: libusb::DeviceHandle<'a>,
    descriptor: libusb::DeviceDescriptor,
    language: Option<libusb::Language>,
    timeout: Duration,
    #[allow(dead_code)]
    max_input_size: u16,
    #[allow(dead_code)]
    in_endpoint_address: u8,
    #[allow(dead_code)]
    out_endpoint_address: u8,
}

impl<'a> RFCatBLDevice<'a> {

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

fn all_rfcatbls(context: &libusb::Context) -> Vec<RFCatBLDevice> {
    let mut rfcatbl_list: Vec<RFCatBLDevice> = Vec::new();
    let devices = match context.devices() {
        Ok(k) => k,
        Err(err) => {
            println!("Error: {}", err);
            return rfcatbl_list;
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
        if is_rfcat_bootloader(&device_desc) {
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
                    // let inbr = interface.number();
                    // println!("i{} kda: {}",
                    //         inbr,
                    //         handle.kernel_driver_active(inbr).unwrap());
                    for interface_desc in interface.descriptors() {
                        // println!("sn{} cc{} scc{} pc{} ne{}",
                        //          interface_desc.setting_number(),
                        //          interface_desc.class_code(),
                        //          interface_desc.sub_class_code(),
                        //          interface_desc.protocol_code(),
                        //          interface_desc.num_endpoints());
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
            // match handle.set_active_configuration(1) {
            //     Ok(k) => k,
            //     Err(err) => {
            //         println!("Error setting configuration: {}", err);
            //         continue
            //     }
            // }
            // match handle.claim_interface(1) {
            //     Ok(k) => k,
            //     Err(err) => {
            //         println!("Error claiming interface: {}", err);
            //         continue
            //     }
            // }
            // match handle.set_alternate_setting(0, 0) {
            //     Ok(k) => k,
            //     Err(err) => {
            //         println!("Error alternate setting: {}", err);
            //         continue
            //     }
            // }
            let exdev = RFCatBLDevice{
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
            rfcatbl_list.push(exdev);
        }
    }
    rfcatbl_list
}

fn main() {
    let matches = App::new("whiskers-bl")
        .version("0.1.0")
        .author("Dave Carlson <thecubic@thecubic.net>")
        .about("RFCat bootloader-mode utility")
        .subcommand(
            SubCommand::with_name("list")
                .about("list attached RFCats in bootloader mode"))
        .subcommand(
            SubCommand::with_name("run")
                .about("exit bootloader mode")
                .arg(Arg::with_name("device")
                    //  .value_name("DEVICE")
                     .help("Bootloader ACM device")
                     .required(true)
                    //  .takes_value(true)
                     .index(1)
        ))
        .get_matches();
    match matches.subcommand_name() {
        Some("list") => {
            let context = libusb::Context::new().unwrap();
            let rfcatbls = all_rfcatbls(&context);
            for rfcatbl in rfcatbls.iter() {
                println!("RFCatBL: b{:03} d{:03} v{:04x} p{:04x}",
                         rfcatbl.bus_number,
                         rfcatbl.address,
                         rfcatbl.vendor_id,
                         rfcatbl.product_id);
                match rfcatbl.manufacturer() {
                    Ok(mstr) => {
                        println!("  {}", mstr);
                    },
                    Err(err) => {
                        println!("  Error: {}", err);
                    },
                }
                match rfcatbl.product() {
                    Ok(pstr) => {
                        println!("  {}", pstr);
                    },
                    Err(err) => {
                        println!("  Error: {}", err);
                    },
                }
            }
        },
        // Some("blrun-lusb") => {
        //     let context = libusb::Context::new().unwrap();
        //     let rfcatbls = all_rfcatbls(&context);
        //     for rfcatbl in rfcatbls.iter() {
        //         println!("RFCatBL: b{:03} d{:03} v{:04x} p{:04x}",
        //                  rfcatbl.bus_number,
        //                  rfcatbl.address,
        //                  rfcatbl.vendor_id,
        //                  rfcatbl.product_id);
        //         match rfcatbl.run() {
        //             Ok(oktho) => {
        //                 println!("  {}", oktho);
        //             },
        //             Err(err) => {
        //                 println!("  Error: {}", err);
        //             },
        //         }
        //     }
        // },
        Some("run") => {
            let subm = matches.subcommand_matches("run").unwrap();
            let devfile = subm.value_of("device").unwrap();
            let mut file = match File::create(Path::new(devfile)) {
                Err(why) => panic!("couldn't open {}: {}",
                                   devfile,
                                   why.description()),
                Ok(file) => file,
            };
            match file.write(":00000001FF\n".as_bytes()) {
                Err(why) => panic!("couldn't write {}: {}",
                                   devfile,
                                   why.description()),
                Ok(_) => (),
            }
        }
        None | _ => (),
    }
}
