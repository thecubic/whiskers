extern crate libusb;
extern crate queues;

use std::slice;
use std::time::Duration;
use std::convert::From;
use std::error::Error;
use std::collections::HashSet;
use std::collections::HashMap;
use queues::Queue;

// #[allow(dead_code)]

#[derive(Copy, Clone, Hash, Eq, PartialEq)]
pub enum SystemCommand {
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
    UNKNOWN = 0x00,
}

// RFST (0xE1) - RF Strobe Commands
pub enum RfState {
    SFSTXON = 0x00,
    SCAL = 0x01,
    SRX = 0x02,
    STX = 0x03,
    SIDLE = 0x04,
    SNOP = 0x05,
    UNKNOWN = 0xFF,
}

impl From<u8> for RfState {
    fn from(value: u8) -> Self {
        match value {
            0x00 => RfState::SFSTXON,
            0x01 => RfState::SCAL,
            0x02 => RfState::SRX,
            0x03 => RfState::STX,
            0x04 => RfState::SIDLE,
            0x05 => RfState::SNOP,
            _ => RfState::UNKNOWN,
        }
    }
}

impl From<u8> for SystemCommand {
    fn from(value: u8) -> Self {
        // this is so boilerplatey There Has To Be Another Way![TM]
        match value {
            0x80 => SystemCommand::Peek,
	        0x81 => SystemCommand::Poke,
	        0x82 => SystemCommand::Ping,
	        0x83 => SystemCommand::Status,
	        0x84 => SystemCommand::PokeRegister,
	        0x85 => SystemCommand::GetClock,
	        0x86 => SystemCommand::BuildType,
	        0x87 => SystemCommand::Bootloader,
	        0x88 => SystemCommand::RFMode,
	        0x89 => SystemCommand::Compiler,
	        0x8e => SystemCommand::PartNum,
	        0x8f => SystemCommand::Reset,
	        0x90 => SystemCommand::ClearCodes,
	        0x93 => SystemCommand::LedMode,
            _ => SystemCommand::UNKNOWN,
        }
    }
}

pub enum Addresses {
    RfState = 0xDFE1,
}

#[derive(Copy, Clone, Hash, Eq, PartialEq)]
pub enum AppMailbox {
	AppGeneric = 0x01,
	AppDebug = 0xfe,
	AppSystem = 0xff,
    UNKNOWN = 0x00,
}

impl From<u8> for AppMailbox {
    fn from(value: u8) -> Self {
        // this is so boilerplatey There Has To Be Another Way![TM]
        match value {
            0x01 => AppMailbox::AppGeneric,
            0xfe => AppMailbox::AppDebug,
            0xff => AppMailbox::AppSystem,
            _ => AppMailbox::UNKNOWN,
        }
    }
}

pub enum CCRegisters {
    Sync1      = 0xdf00,
    Sync0      = 0xdf01,
    PktLen     = 0xdf02,
    PktCtrl1   = 0xdf03,
    PktCtrl0   = 0xdf04,
    Addr       = 0xdf05,
    ChanNr     = 0xdf06,
    FsCtrl1    = 0xdf07,
    FsCtrl0    = 0xdf08,
    Freq2      = 0xdf09,
    Freq1      = 0xdf0a,
    Freq0      = 0xdf0b,
    MdmCfg4    = 0xdf0c,
    MdmCfg3    = 0xdf0d,
    MdmCfg2    = 0xdf0e,
    MdmCfg1    = 0xdf0f,
    MdmCfg0    = 0xdf10,
    Deviatn    = 0xdf11,
    Mcsm2      = 0xdf12,
    Mcsm1      = 0xdf13,
    Mcsm0      = 0xdf14,
    FocCfg     = 0xdf15,
    BsCfg      = 0xdf16,
    AgcCtrl2   = 0xdf17,
    AgcCtrl1   = 0xdf18,
    AgcCtrl0   = 0xdf19,
    FrEnd1     = 0xdf1a,
    FrEnd0     = 0xdf1b,
    FsCal3     = 0xdf1c,
    FsCal2     = 0xdf1d,
    FsCal1     = 0xdf1e,
    FsCal0     = 0xdf1f,
    Z0         = 0xdf20,
    Z1         = 0xdf21,
    Z2         = 0xdf22,
    Test2      = 0xdf23,
    Test1      = 0xdf24,
    Test0      = 0xdf25,
    Z3         = 0xdf26,
    PaTable7  = 0xdf27,
    PaTable6  = 0xdf28,
    PaTable5  = 0xdf29,
    PaTable4  = 0xdf2a,
    PaTable3  = 0xdf2b,
    PaTable2  = 0xdf2c,
    PaTable1  = 0xdf2d,
    PaTable0  = 0xdf2e,
    IoCfg2     = 0xdf2f,
    IoCfg1     = 0xdf30,
    IoCfg0     = 0xdf31,
    Z4         = 0xdf32,
    Z5         = 0xdf33,
    Z6         = 0xdf34,
    Z7         = 0xdf35,
    PartNum    = 0xdf36,
    ChipId     = 0xdf37,
    FreqEst    = 0xdf38,
    Lqi        = 0xdf39,
    Rssi       = 0xdf3a,
    MarcState  = 0xdf3b,
    PkStatus   = 0xdf3c,
    VcoVcDac = 0xdf3d,
}


pub struct RFCatDevice<'a> {
    pub bus_number: u8,
    pub address: u8,
    pub vendor_id: u16,
    pub product_id: u16,
    handle: libusb::DeviceHandle<'a>,
    descriptor: libusb::DeviceDescriptor,
    language: Option<libusb::Language>,
    timeout: Duration,
    max_input_size: u16,
    in_endpoint_address: u8,
    out_endpoint_address: u8,
    //
    radio_mode: Option<RfState>,
    mailbox_queues: HashMap<(AppMailbox, SystemCommand), Queue<RfCatPacket>>,
}

#[derive(Clone)]
pub struct RfCatPacket {
    pub cmd: SystemCommand,
    pub mbx: AppMailbox,
    pub payload: Vec<u8>,
    pub received: bool,
}

impl<'a> RfCatPacket {
    pub fn simple(mbx: AppMailbox, cmd: SystemCommand) -> RfCatPacket {
        RfCatPacket{mbx: mbx, cmd: cmd, payload: Vec::<u8>::new(), received: false}
    }
    pub fn payload(mbx: AppMailbox, cmd: SystemCommand, payload: Vec<u8>) -> RfCatPacket {
        RfCatPacket{mbx: mbx, cmd: cmd, payload: payload, received: false}
    }

    pub fn from_bytes(bytes: Vec<u8>) -> RfCatPacket {
        let plen: u16 = u16::from_le_bytes([bytes[3], bytes[4]]);
        let mut rcvr: Vec<u8> = Vec::<u8>::with_capacity(plen as usize);
        let payload_end: u16 = 5 + plen;
        match payload_end {
            0 => (),
            _ => {
                for offset in 5..payload_end {
                    rcvr.push(bytes[offset as usize]);
                }
            }
        }
        RfCatPacket{mbx: AppMailbox::from(bytes[1]), cmd: SystemCommand::from(bytes[2]), payload: rcvr, received: true}
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut pktbytes = Vec::<u8>::with_capacity(self.payload.len() + 4);
        pktbytes.push(self.mbx as u8);
        pktbytes.push(self.cmd as u8);
        for b in (self.payload.len() as u16).to_le_bytes().iter() {
            pktbytes.push(*b);
        }
        for b in self.payload.iter() {
            pktbytes.push(*b);
        }
        pktbytes
    }
}

impl<'a> RFCatDevice<'a> {

    /* USB layer */
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

    /* CC layer */

    /* send a command packet to the CC down the wire(s) */
    pub fn mail(&self, pkt: RfCatPacket) -> Result<usize, libusb::Error> {
        self.handle.write_bulk(self.out_endpoint_address, &pkt.to_bytes()[..], self.timeout)
    }

    /* raw, un-mailboxed receive */
    pub fn recv(&self) -> Result<RfCatPacket, libusb::Error> {
        let mut in_vec = Vec::<u8>::with_capacity(self.max_input_size as usize);
        let in_buf = unsafe { slice::from_raw_parts_mut((&mut in_vec[..]).as_mut_ptr(), in_vec.capacity()) };
        match self.handle.read_bulk(self.in_endpoint_address, in_buf, self.timeout) {
            Ok(rlen) => {
                unsafe { in_vec.set_len(rlen) };
                return Ok(RfCatPacket::from_bytes(in_vec));
            },
            Err(err) => {
                println!("nope: {}", err);
                return Err(err);
            },
        }
    }

    /* simple CC communication with no payload */
    pub fn ping(&self) -> Result<bool, libusb::Error> {
        match self.mail(RfCatPacket::simple(AppMailbox::AppSystem, SystemCommand::Ping)) {
            Ok(_) => (),
            Err(err) => {
                return Err(err);
            },
        }
        match self.recv() {
            Ok(pkt) => {
                return Ok(true);
            },
            Err(err) => {
                return Err(err);
            },
        }
    }

    /* tell the CC to escape into bootloader mode (requires CC bootloader) */
    pub fn bootloader(&self) -> Result<bool, libusb::Error> {
        match self.mail(RfCatPacket::simple(AppMailbox::AppSystem, SystemCommand::Bootloader)) {
            Ok(_) => (),
            Err(err) => {
                return Err(err);
            },
        }
        match self.recv() {
            Ok(pkt) => {
                return Ok(true);
            },
            Err(err) => {
                return Err(err);
            },
        }
    }

    /* retrieve the CC firmware's build name if present (e.g. YARDSTICKONE r0543) */
    pub fn buildname(&self) -> Result<Option<String>, libusb::Error> {
        match self.mail(RfCatPacket::simple(AppMailbox::AppSystem, SystemCommand::BuildType)) {
            Ok(_) => (),
            Err(err) => {
                return Err(err);
            },
        }
        match self.recv() {
            Ok(pkt) => {
                if pkt.payload.len() > 0 {
                    return Ok(Some(String::from_utf8(pkt.payload).unwrap()));
                } else {
                    return Ok(None);
                }
            },
            Err(err) => {
                return Err(err);
            },
        }
    }

    /* retrieve the CC firmware's compiler name if present (e.g. SDCCv370) */
    pub fn compiler(&self) -> Result<Option<String>, libusb::Error> {
        match self.mail(RfCatPacket::simple(AppMailbox::AppSystem, SystemCommand::Compiler)) {
            Ok(_) => (),
            Err(err) => {
                return Err(err);
            },
        }
        match self.recv() {
            Ok(pkt) => {
                if pkt.payload.len() > 0 {
                    return Ok(Some(String::from_utf8(pkt.payload).unwrap()));
                } else {
                    return Ok(None);
                }
            },
            Err(err) => {
                return Err(err);
            },
        }
    }

    pub fn reset(&self) -> Result<usize, libusb::Error> {
        self.mail(RfCatPacket::payload(AppMailbox::AppSystem,
                                       SystemCommand::Reset, 
                                       "RESET_NOW\x00".as_bytes().to_vec()))
    }

    pub fn peek(&self, addr: u16, bytecount: u16) -> Result<Vec<u8>, libusb::Error> {
        // TODO: this is reeeal stupid
        let bcle = bytecount.to_le_bytes();
        let adle = addr.to_le_bytes();

        match self.mail(RfCatPacket::payload(AppMailbox::AppSystem,
                                             SystemCommand::Peek,
                                             vec![bcle[0], bcle[1], adle[0], adle[1]])) {
            Ok(_) => (),
            Err(err) => {
                return Err(err);
            },
        }
        match self.recv() {
            Ok(pkt) => {
                return Ok(pkt.payload.to_owned());
            },
            Err(err) => {
                return Err(err);
            },
        }
    }

    pub fn has_bootloader(&self) -> Result<bool, libusb::Error> {
        // SFR I2SCLKF0 & I2SCLKF1
        match self.peek(0xDF46, 2) {
            Ok(magic) => {
                // 0x0DF0 ? I don't get it
                return Ok(magic[0] == 0xF0 && magic[1] == 0x0D);
            },
            Err(err) => {
                return Err(err);
            },
        }
    }

    // HERE BE DRAGONS
    // OH NO DON'T SHOW GITHUB MY DRAGONS

    // [DRAGONS REDACTED]

    pub fn poke(&self, addr: u16, data: u8) -> Result<(), libusb::Error> {
        // self.push(AppMailbox::AppSystem, SystemCommand::Poke, vec![addr.to_le_bytes(), data]);
        Ok(())
    }

    pub fn poke_reg(&self, addr: u16, data: u8) -> Result<(), libusb::Error> {
        // self.push(AppMailbox::AppSystem, SystemCommand::PokeRegister, vec!([addr.to_le_bytes(), data]));
        Ok(())
    }     

    pub fn get_interrupt_registers(&self) -> Result<(), libusb::Error> {
        Ok(())
    }


    pub fn send(&self, mbx: AppMailbox, cmd: SystemCommand) -> Result<(), libusb::Error> {
        Ok(())
    }

    pub fn push(&self, mbx: AppMailbox, cmd: SystemCommand, payload: Vec<u8>) -> Result<(), libusb::Error> {
        Ok(())
    }


    pub fn set_rf_mode(&self, rfmode: RfState) {
        // self.currentRfMode = rfmode;
        self.push(AppMailbox::AppSystem, SystemCommand::RFMode, vec![rfmode as u8]);
    }

    //     ### set standard radio state to TX/RX/IDLE (TX is pretty much only good for jamming).  TX/RX modes are set to return to whatever state you choose here.
    pub fn set_mode_tx(&self) {
        //         BOTH: set radio to TX state
        //         AND:  set radio to return to TX state when done with other states
        self.set_rf_mode(RfState::STX);
    }
            
    pub fn set_mode_rx(&self) {
        //         BOTH: set radio to RX state
        //         AND:  set radio to return to RX state when done with other states
        self.set_rf_mode(RfState::SRX);
    }

    pub fn set_mode_idle(&self) {
        //         BOTH: set radio to IDLE state
        //         AND:  set radio to return to IDLE state when done with other states
        self.set_rf_mode(RfState::SIDLE);
    }

    pub fn strobe_rf_mode(&self, rfmode: RfState) {
        self.poke(Addresses::RfState as u16, rfmode as u8);
    }

    //     ### send raw state change to radio (doesn't update the return state for after RX/TX occurs)
    pub fn strobe_mode_tx(&self) {
        //         set radio to TX state (transient)
        self.strobe_rf_mode(RfState::STX);
    }

    pub fn strobe_mode_rx(&self) {
        //         set radio to RX state (transient)
        self.strobe_rf_mode(RfState::SRX);
    }

    pub fn strobe_mode_idle(&self) {
        //         set radio to IDLE state (transient)
        self.strobe_rf_mode(RfState::SIDLE);
    }

    pub fn strobe_mode_fstxon(&self) {
        //         set radio to FSTXON state (transient)
        self.strobe_rf_mode(RfState::SFSTXON);
    }

    pub fn strobe_mode_cal(&self) {
        //         set radio to CAL state (will return to whichever state is configured (via setMode* pub fntions)
        self.strobe_rf_mode(RfState::SCAL);
    }

    pub fn strobe_mode_return(&self) {
    //         attempts to return the the correct mode after configuring some radio register(s).
    //         it uses the marcstate provided (or self.radiocfg.marcstate if none are provided) to determine how to strobe the radio.
    //         #if marcstate is None:
    //             #marcstate = self.radiocfg.marcstate
    //         #if self._debug: print("MARCSTATE: %x   returning to %x" % (marcstate, MARC_STATE_MAPPINGS[marcstate][2]) )
    //         #self.poke(X_RFST, "%c"%MARC_STATE_MAPPINGS[marcstate][2])
    //         self.poke(X_RFST, "%c" % self._rfmode)
    }

    pub fn get_frequency(&self) -> u64 {
        let freq: u64 = 0;

        freq
    }
        

    pub fn get_radio_config(&self) -> Result<RadioConfig, libusb::Error> {
        match self.peek(0xdf00, 0x3e) {
            Ok(data) => Ok(RadioConfig{}),
            Err(err) => Err(err),
        }
    }

    pub fn make_from_libusb(
        device: libusb::Device,
        device_desc: libusb::DeviceDescriptor,
    ) -> Result<RFCatDevice, libusb::Error> {
        let mut handle = match device.open() {
            Ok(k) => k,
            Err(err) => {
                println!("Error opening device: {}", err);
                return Err(err);
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
                return Err(err);
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
                return Err(err);
            }
        };
        for n in 0..device_desc.num_configurations() {
            let config_desc = match device.config_descriptor(n) {
                Ok(k) => k,
                Err(err) => {
                    println!("Error getting config descriptor: {}", err);
                    return Err(err);
                }
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
                return Err(err);
            }
        }
        match handle.claim_interface(0) {
            Ok(k) => k,
            Err(err) => {
                println!("Error claiming interface: {}", err);
                return Err(err);
            }
        }
        match handle.set_alternate_setting(0, 0) {
            Ok(k) => k,
            Err(err) => {
                println!("Error alternate setting: {}", err);
                return Err(err);
            }
        }
        Ok(RFCatDevice{
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
            radio_mode: None,
            mailbox_queues: HashMap::new(),
        })
    }
}

pub struct RadioConfig {

}
impl RadioConfig {
    fn from_bytes(v: Vec<u8>) -> Result<RadioConfig, libusb::Error> {
        // deserialize here
        Ok(RadioConfig{})
    }
    fn to_bytes(&self) -> Result<Vec<u8>, libusb::Error> {
        Ok(vec![0])
    }
}
    


// TODO: this should be a static vector for vps
fn is_standard_rfcat(usbdd: &libusb::DeviceDescriptor) -> bool {
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

fn parse_two_hex(twohex: &str) -> (u16, u16) {
    let parts: Vec<&str> = twohex.split(",").collect();
    (u16::from_str_radix(parts[0], 16).unwrap(),
     u16::from_str_radix(parts[1], 16).unwrap())
}

fn parse_two_ints(twoints: &str) -> (u8, u8) {
    let parts: Vec<&str> = twoints.split(",").collect();
    (parts[0].parse::<u8>().unwrap(), parts[1].parse::<u8>().unwrap())
}

fn parse_vp(vps: Vec<&str>) -> HashSet<(u16, u16)> {
    let mut vpset = HashSet::<(u16, u16)>::new();
    for vp in vps.iter() {
        vpset.insert(parse_two_hex(vp));
    }
    vpset
}

fn parse_addrs(addrs: Vec<&str>) -> HashSet<(u8, u8)> {
    let mut addrset = HashSet::<(u8, u8)>::new();
    for addr in addrs.iter() {
        addrset.insert(parse_two_ints(addr));
    }
    addrset
}

pub fn rfcat_filter<'a>(
    usb_context: Option<&'a libusb::Context>,
    usb_addresses: Option<Vec<&str>>,
    usb_vendor_products: Option<Vec<&str>>,
    /* TODO: SPI et al */
) -> Vec<RFCatDevice<'a>> {
    let mut rfcat_list: Vec<RFCatDevice> = Vec::new();
    let mut picked_addresses: bool;
    let mut addresses: HashSet<(u8, u8)>;
    match usb_addresses {
        None => {
            picked_addresses = false;
            addresses = HashSet::<(u8, u8)>::new();
        }
        Some(addrs) => {
            picked_addresses = true;
            addresses = parse_addrs(addrs);
        }
    }

    let mut picked_vps: bool;
    let mut vps: HashSet<(u16, u16)>;
    match usb_vendor_products {
        None => {
            picked_vps = false;
            // TODO: the standard ones
            vps = HashSet::<(u16,u16)>::new();
        }
        Some(uvps) => {
            picked_vps = true;
            vps = parse_vp(uvps);
        }
    }
    if let Some(ctx) = usb_context {
        let usb_devices = match ctx.devices() {
            Ok(devs) => devs,
            Err(err) => {
                println!("Error: {}", err);
                return rfcat_list;
            }
        };
        for device in usb_devices.iter() {
            let device_desc = match device.device_descriptor() {
                Ok(k) => k,
                Err(err) => {
                    println!("Error getting descriptor: {}", err);
                    continue
                }
            };
            /* Address matching */
            if picked_addresses {
                /* provided addresses */
                if !addresses.contains(&(device.bus_number(), device.address())) {
                    continue
                }
            }
            /* Vendor / Product matching */
            if picked_vps {
                /* provided vendor / products */
                if !vps.contains(&(device_desc.vendor_id(), device_desc.product_id())) {
                    continue
                }
            } else {
                /* standard */
                if !is_standard_rfcat(&device_desc) {
                    continue
                }
            }
            match RFCatDevice::make_from_libusb(device, device_desc) {
                Ok(rfcat_dev) => { rfcat_list.push(rfcat_dev) },
                Err(_) => {},
            }
        }
    }
    rfcat_list
}

pub struct RFCatBLDevice<'a> {
    pub bus_number: u8,
    pub address: u8,
    pub vendor_id: u16,
    pub product_id: u16,
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

pub fn all_rfcatbls(context: &libusb::Context) -> Vec<RFCatBLDevice> {
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

// BEGIN dogscience and copypasting

// enum BandLimitsMHz {
//     B300 = (281, 361),
//     B400 = (378, 749),
//     B900 = (749, 962),
// }

enum BandTransitionsMHz {
    BT400 = 369,
    BT900 = 615,
}

enum VCOTransitionsMHz {
    VT300 = 318,
    VT400 = 424,
    VT900 = 848,
}

enum SyncM {
    SMNone = 0,
    SM15of16 = 1,
    SM16of16 = 2,
    SM30of32 = 7,
}

// 0xDF3B: MARCSTATE - Main Radio Control State Machine State
// #define MARCSTATE_MARC_STATE              0x1F

enum MainRadioControlState {
    Sleep                  = 0x00,
    Idle                   = 0x01,
    VcoOnMc               = 0x03,
    RegOnMc               = 0x04,
    ManCal                 = 0x05,
    VcoOn                  = 0x06,
    RegOn                  = 0x07,
    StartCal               = 0x08,
    BwBoost                = 0x09,
    FsLock                = 0x0A,
    IfadCon                = 0x0B,
    EndCal                 = 0x0C,
    Rx                     = 0x0D,
    RxEnd                 = 0x0E,
    RxRst                 = 0x0F,
    TxRxSwitch            = 0x10,
    RxOverflow            = 0x11,
    FstXOn                 = 0x12,
    Tx                     = 0x13,
    TxEnd                 = 0x14,
    RxTxSwitch            = 0x15,
    TxUnderflow           = 0x16,
}

// 0xDF3C Packetstatus register
enum PacketStatusRegister {
    SFD                     = 0x08,
    CCA                     = 0x10,
    PQTREACHED              = 0x20,
    CS                      = 0x40,
    CRCOK                   = 0x80,
}

