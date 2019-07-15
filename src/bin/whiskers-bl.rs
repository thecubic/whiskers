extern crate libusb;
extern crate clap;

// use std::slice;
use std::time::Duration;
use clap::{App, Arg, SubCommand};
use std::fs::File;
use std::path::Path;
use std::error::Error;
use std::io::Write;

use whiskers::RFCatBLDevice;
use whiskers::all_rfcatbls;


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
