extern crate libusb;
extern crate clap;

use clap::{App, SubCommand, Arg, ArgGroup};
use whiskers::rfcat_filter;
use std::time::Instant;

fn main() {
    let matches = App::new("whiskers")
        .version("0.2.0")
        .author("Dave Carlson <thecubic@thecubic.net>")
        .about("RFCat driver application")
        .subcommand(
            SubCommand::with_name("list")
                .about("list USB-attached RFCats"))
        .subcommand(
            SubCommand::with_name("buildname")
                .about("display the build name")
                .arg(Arg::with_name("usb-vp")
                    .help("select specific USB vendor & product combos (format: <vendor hex>,<product hex>)")
                    .long("usb-vp")
                    .takes_value(true)
                    .multiple(true)
                    .required(false))
                .group(ArgGroup::with_name("usb-select")
                    .required(true)
                    .arg("all-usb")
                    .arg("usb-addr"))
                .arg(Arg::with_name("usb-addr")
                    .help("select specific USB addresses (format: <bus nbr>,<dev nbr>)")
                    .long("usb-addr")
                    .takes_value(true)
                    .multiple(true)
                    .required(false))
                .arg(Arg::with_name("all-usb")
                    .help("select all USB addresses")
                    .long("all-usb")
                    .long("usb-all")
                    .required(false)))
        .subcommand(
            SubCommand::with_name("compiler")
                .about("display the compiler info")
                .arg(Arg::with_name("usb-vp")
                    .help("select specific USB vendor & product combos (format: <vendor hex>,<product hex>)")
                    .long("usb-vp")
                    .takes_value(true)
                    .multiple(true)
                    .required(false))
                .group(ArgGroup::with_name("usb-select")
                    .required(true)
                    .arg("all-usb")
                    .arg("usb-addr"))
                .arg(Arg::with_name("usb-addr")
                    .help("select specific USB addresses (format: <bus nbr>,<dev nbr>)")
                    .long("usb-addr")
                    .takes_value(true)
                    .multiple(true)
                    .required(false))
                .arg(Arg::with_name("all-usb")
                    .help("select all USB addresses")
                    .long("all-usb")
                    .long("usb-all")
                    .required(false)))
        .subcommand(
            SubCommand::with_name("bootloader")
                .about("place rfcats in bootloader mode")
                .arg(Arg::with_name("usb-vp")
                    .help("select specific USB vendor & product combos (format: <vendor hex>,<product hex>)")
                    .long("usb-vp")
                    .takes_value(true)
                    .multiple(true)
                    .required(false))
                .group(ArgGroup::with_name("usb-select")
                    .required(true)
                    .arg("all-usb")
                    .arg("usb-addr"))
                .arg(Arg::with_name("usb-addr")
                    .help("select specific USB addresses (format: <bus nbr>,<dev nbr>)")
                    .long("usb-addr")
                    .takes_value(true)
                    .multiple(true)
                    .required(false))
                .arg(Arg::with_name("all-usb")
                    .help("select all USB addresses")
                    .long("all-usb")
                    .long("usb-all")
                    .required(false)))
        .subcommand(
            SubCommand::with_name("ping")
                .about("ping the device(s)")
                .arg(Arg::with_name("usb-vp")
                    .help("select specific USB vendor & product combos (format: <vendor hex>,<product hex>)")
                    .long("usb-vp")
                    .takes_value(true)
                    .multiple(true)
                    .required(false))
                .group(ArgGroup::with_name("usb-select")
                    .required(true)
                    .arg("all-usb")
                    .arg("usb-addr"))
                .arg(Arg::with_name("usb-addr")
                    .help("select specific USB addresses (format: <bus nbr>,<dev nbr>)")
                    .long("usb-addr")
                    .takes_value(true)
                    .multiple(true)
                    .required(false))
                .arg(Arg::with_name("all-usb")
                    .help("select all USB addresses")
                    .long("all-usb")
                    .long("usb-all")
                    .required(false)))
        .subcommand(
            SubCommand::with_name("peektest")
                .about("peek the device(s)")
                .arg(Arg::with_name("usb-vp")
                    .help("select specific USB vendor & product combos (format: <vendor hex>,<product hex>)")
                    .long("usb-vp")
                    .takes_value(true)
                    .multiple(true)
                    .required(false))
                .group(ArgGroup::with_name("usb-select")
                    .required(true)
                    .arg("all-usb")
                    .arg("usb-addr"))
                .arg(Arg::with_name("usb-addr")
                    .help("select specific USB addresses (format: <bus nbr>,<dev nbr>)")
                    .long("usb-addr")
                    .takes_value(true)
                    .multiple(true)
                    .required(false))
                .arg(Arg::with_name("all-usb")
                    .help("select all USB addresses")
                    .long("all-usb")
                    .long("usb-all")
                    .required(false)))
        .subcommand(
            SubCommand::with_name("has-bootloader")
                .about("tests for CC-Bootloader")
                .arg(Arg::with_name("usb-vp")
                    .help("select specific USB vendor & product combos (format: <vendor hex>,<product hex>)")
                    .long("usb-vp")
                    .takes_value(true)
                    .multiple(true)
                    .required(false))
                .group(ArgGroup::with_name("usb-select")
                    .required(true)
                    .arg("all-usb")
                    .arg("usb-addr"))
                .arg(Arg::with_name("usb-addr")
                    .help("select specific USB addresses (format: <bus nbr>,<dev nbr>)")
                    .long("usb-addr")
                    .takes_value(true)
                    .multiple(true)
                    .required(false))
                .arg(Arg::with_name("all-usb")
                    .help("select all USB addresses")
                    .long("all-usb")
                    .long("usb-all")
                    .required(false)))                
        .get_matches();
    match matches.subcommand_name() {
        
        Some("buildname") => {
            let argm = matches.subcommand_matches("buildname").unwrap();
            /* TODO: for SPI-et-al support, USB must be optional */
            let context = libusb::Context::new().unwrap();
            
            let usb_vp = match argm.is_present("usb-vp") {
                false => None,
                true => Some(argm.values_of("usb-vp").unwrap().collect()),
            };

            let usb_addr = match argm.is_present("usb-addr") {
                false => None,
                true => Some(argm.values_of("usb-addr").unwrap().collect()),
            };

            let rfcats = rfcat_filter(Some(&context), usb_addr, usb_vp);
            for rfcat in rfcats.iter() {
                println!("RFCat: b{:03} d{:03}d v{:04x} p{:04x}",
                         rfcat.bus_number,
                         rfcat.address,
                         rfcat.vendor_id,
                         rfcat.product_id);
                match rfcat.buildname() {
                    Ok(Some(buildname)) => {println!("  buildname: {}", buildname)},
                    Ok(None) => {println!("  no-buildname")},
                    Err(err) => {println!("  Error: {}", err)},
                }
            }
        },
        Some("compiler") => {
            let argm = matches.subcommand_matches("compiler").unwrap();
            /* TODO: for SPI-et-al support, USB must be optional */
            let context = libusb::Context::new().unwrap();
            
            let usb_vp = match argm.is_present("usb-vp") {
                false => None,
                true => Some(argm.values_of("usb-vp").unwrap().collect()),
            };

            let usb_addr = match argm.is_present("usb-addr") {
                false => None,
                true => Some(argm.values_of("usb-addr").unwrap().collect()),
            };

            let rfcats = rfcat_filter(Some(&context), usb_addr, usb_vp);
            for rfcat in rfcats.iter() {
                println!("RFCat: b{:03} d{:03}d v{:04x} p{:04x}",
                         rfcat.bus_number,
                         rfcat.address,
                         rfcat.vendor_id,
                         rfcat.product_id);
                match rfcat.compiler() {
                    Ok(Some(compiler)) => {println!("  compiler: {}", compiler)},
                    Ok(None) => {println!("  no-compiler")},
                    Err(err) => {println!("  Error: {}", err)},
                }
            }
        },
        Some("bootloader") => {
            let argm = matches.subcommand_matches("ping").unwrap();
            /* TODO: for SPI-et-al support, USB must be optional */
            let context = libusb::Context::new().unwrap();
            
            let usb_vp = match argm.is_present("usb-vp") {
                false => None,
                true => Some(argm.values_of("usb-vp").unwrap().collect()),
            };

            let usb_addr = match argm.is_present("usb-addr") {
                false => None,
                true => Some(argm.values_of("usb-addr").unwrap().collect()),
            };

            let rfcats = rfcat_filter(Some(&context), usb_addr, usb_vp);
            for rfcat in rfcats.iter() {
                println!("RFCat: b{:03} d{:03} v{:04x} p{:04x}",
                         rfcat.bus_number,
                         rfcat.address,
                         rfcat.vendor_id,
                         rfcat.product_id);
                match rfcat.bootloader() {
                    Ok(oktho) => {
                        println!("  {}", oktho);
                    },
                    Err(err) => {
                        println!("  Error: {}", err);
                    },
                }
            }
        },
        Some("ping") => {
            let argm = matches.subcommand_matches("ping").unwrap();
            /* TODO: for SPI-et-al support, USB must be optional */
            let context = libusb::Context::new().unwrap();
            
            let usb_vp = match argm.is_present("usb-vp") {
                false => None,
                true => Some(argm.values_of("usb-vp").unwrap().collect()),
            };

            let usb_addr = match argm.is_present("usb-addr") {
                false => None,
                true => Some(argm.values_of("usb-addr").unwrap().collect()),
            };

            let rfcats = rfcat_filter(Some(&context), usb_addr, usb_vp);

            for rfcat in rfcats.iter() {
                let pre = Instant::now();
                println!("RFCat: b{:03} d{:03} v{:04x} p{:04x}",
                         rfcat.bus_number,
                         rfcat.address,
                         rfcat.vendor_id,
                         rfcat.product_id);
                match rfcat.ping() {
                    Ok(oktho) => {
                        println!("  {} ({} us)", oktho, pre.elapsed().as_micros());
                    },
                    Err(err) => {
                        println!("  Error: {}", err);
                    },
                }
            }
        },
        Some("peektest") => {
            let argm = matches.subcommand_matches("peek").unwrap();
            /* TODO: for SPI-et-al support, USB must be optional */
            let context = libusb::Context::new().unwrap();
            
            let usb_vp = match argm.is_present("usb-vp") {
                false => None,
                true => Some(argm.values_of("usb-vp").unwrap().collect()),
            };

            let usb_addr = match argm.is_present("usb-addr") {
                false => None,
                true => Some(argm.values_of("usb-addr").unwrap().collect()),
            };

            let rfcats = rfcat_filter(Some(&context), usb_addr, usb_vp);

            for rfcat in rfcats.iter() {
                let pre = Instant::now();
                println!("RFCat: b{:03} d{:03} v{:04x} p{:04x}",
                         rfcat.bus_number,
                         rfcat.address,
                         rfcat.vendor_id,
                         rfcat.product_id);
                match rfcat.peek(0xDF46, 2) {
                    Ok(data) => {
                        println!("  {} ({} us)", data.len(), pre.elapsed().as_micros());
                        if data.len() == 2 {
                            println!("  {} {}", data[0], data[1]);
                        }
                    },
                    Err(err) => {
                        println!("  Error: {}", err);
                    },
                }
            }
        },
        Some("has-bootloader") => {
            let argm = matches.subcommand_matches("has-bootloader").unwrap();
            /* TODO: for SPI-et-al support, USB must be optional */
            let context = libusb::Context::new().unwrap();
            
            let usb_vp = match argm.is_present("usb-vp") {
                false => None,
                true => Some(argm.values_of("usb-vp").unwrap().collect()),
            };

            let usb_addr = match argm.is_present("usb-addr") {
                false => None,
                true => Some(argm.values_of("usb-addr").unwrap().collect()),
            };

            let rfcats = rfcat_filter(Some(&context), usb_addr, usb_vp);

            for rfcat in rfcats.iter() {
                let pre = Instant::now();
                println!("RFCat: b{:03} d{:03} v{:04x} p{:04x}",
                         rfcat.bus_number,
                         rfcat.address,
                         rfcat.vendor_id,
                         rfcat.product_id);
                match rfcat.has_bootloader() {
                    Ok(bootloader) => {
                        println!("  {} ({} us)", bootloader, pre.elapsed().as_micros());
                    },
                    Err(err) => {
                        println!("  Error: {}", err);
                    },
                }
            }
        },
        None | _ => (),
    }
}
