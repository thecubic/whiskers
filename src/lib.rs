extern crate libusb;

use std::slice;
use std::time::Duration;

#[allow(dead_code)]
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
}

#[allow(dead_code)]
pub enum AppMailbox {
	AppGeneric = 0x01,
	AppDebug = 0xfe,
	AppSystem = 0xff,
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
}

pub struct RfCatPacket {
    pub cmd: SystemCommand,
    pub mbx: AppMailbox,
}

impl<'a> RFCatDevice<'a> {
    pub fn buildname(&self) -> Result<String, libusb::Error> {
        let mut in_vec = Vec::<u8>::with_capacity(self.max_input_size as usize);
        let in_buf = unsafe { slice::from_raw_parts_mut((&mut in_vec[..]).as_mut_ptr(), in_vec.capacity()) };
        // TODO: packet builder
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

    pub fn compiler(&self) -> Result<String, libusb::Error> {
        let mut in_vec = Vec::<u8>::with_capacity(self.max_input_size as usize);
        let in_buf = unsafe { slice::from_raw_parts_mut((&mut in_vec[..]).as_mut_ptr(), in_vec.capacity()) };
        // TODO: packet builder
        let outvec = vec![AppMailbox::AppSystem as u8,
                          SystemCommand::Compiler as u8,
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
                assert_eq!(in_vec[2], SystemCommand::Compiler as u8);
                let slen = u16::from_le_bytes([in_vec[3], in_vec[4]]);
                println!(" slen: {}", slen);
                if slen == 0 {
                    return Ok("0-length".to_string());
                }
                let compiler = String::from_utf8(in_vec[5..4+(slen as usize)].to_vec()).unwrap();
                return Ok(compiler);
            },
            Err(err) => {
                println!("nope: {}", err);
                return Err(err);
            }
        }
    }



//         def RESET(self):
//         try:
//             r = self.send(APP_SYSTEM, SYS_CMD_RESET, "RESET_NOW\x00")
//         except ChipconUsbTimeoutException:
//             pass
        
//     def peek(self, addr, bytecount=1):
//         r, t = self.send(APP_SYSTEM, SYS_CMD_PEEK, struct.pack("<HH", bytecount, addr))
//         return r

//     def poke(self, addr, data):
//         r, t = self.send(APP_SYSTEM, SYS_CMD_POKE, struct.pack("<H", addr) + data)
//         return r
    
//     def pokeReg(self, addr, data):
//         r, t = self.send(APP_SYSTEM, SYS_CMD_POKE_REG, struct.pack("<H", addr) + data)
//         return r

//     def getBuildInfo(self):
//         r, t = self.send(APP_SYSTEM, SYS_CMD_BUILDTYPE, '')
//         return r
            
//     def getCompilerInfo(self):
//         r, t = self.send(APP_SYSTEM, SYS_CMD_COMPILER, '')
//         return r
            
//     def getInterruptRegisters(self):
//         regs = {}
//         # IEN0,1,2
//         regs['IEN0'] = self.peek(IEN0,1)
//         regs['IEN1'] = self.peek(IEN1,1)
//         regs['IEN2'] = self.peek(IEN2,1)
//         # TCON
//         regs['TCON'] = self.peek(TCON,1)
//         # S0CON
//         regs['S0CON'] = self.peek(S0CON,1)
//         # IRCON
//         regs['IRCON'] = self.peek(IRCON,1)
//         # IRCON2
//         regs['IRCON2'] = self.peek(IRCON2,1)
//         # S1CON
//         regs['S1CON'] = self.peek(S1CON,1)
//         # RFIF
//         regs['RFIF'] = self.peek(RFIF,1)
//         # DMAIE
//         regs['DMAIE'] = self.peek(DMAIE,1)
//         # DMAIF
//         regs['DMAIF'] = self.peek(DMAIF,1)
//         # DMAIRQ
//         regs['DMAIRQ'] = self.peek(DMAIRQ,1)
//         return regs

//     def reprHardwareConfig(self):
//         output= []

//         hardware = self.getBuildInfo()
//         output.append("Dongle:              %s" % hardware.split(' ')[0])
//         try:
//             output.append("Firmware rev:        %s" % hardware.split('r')[1])
//         except:
//             output.append("Firmware rev:        Not found! Update needed!")
//         try:
//             compiler = self.getCompilerInfo()
//             output.append("Compiler:            %s" % compiler)
//         except:
//             output.append("Compiler:            Not found! Update needed!")
//         # see if we have a bootloader by loooking for it's recognition semaphores
//         # in SFR I2SCLKF0 & I2SCLKF1
//         if(self.peek(0xDF46,1) == '\xF0' and self.peek(0xDF47,1) == '\x0D'):
//             output.append("Bootloader:          CC-Bootloader")
//         else:
//             output.append("Bootloader:          Not installed")
//         return "\n".join(output)

//     def reprSoftwareConfig(self):
//         output= []

//         output.append("rflib rev:           %s" % RFLIB_VERSION)
//         return "\n".join(output)

//     def printClientState(self, width=120):
//         print(self.reprClientState(width))

//     def reprClientState(self, width=120):
//         output = ["="*width]
//         output.append('     client thread cycles:      %d/%d' % (self.recv_threadcounter,self.send_threadcounter))
//         output.append('     client errored cycles:     %d' % self._usberrorcnt)
//         output.append('     recv_queue:                (%d bytes) %s'%(len(self.recv_queue),repr(self.recv_queue)[:width-42]))
//         output.append('     trash:                     (%d blobs) "%s"'%(len(self.trash),repr(self.trash)[:width-44]))
//         output.append('     recv_mbox                  (%d keys)  "%s"'%(len(self.recv_mbox),repr([hex(x) for x in list(self.recv_mbox.keys())])[:width-44]))
//         for app in list(self.recv_mbox.keys()):
//             appbox = self.recv_mbox[app]
//             output.append('       app 0x%x (%d records)'%(app,len(appbox)))
//             for cmd in list(appbox.keys()):
//                 output.append('             [0x%x]    (%d frames)  "%s"'%(cmd, len(appbox[cmd]), repr(appbox[cmd])[:width-36]))
//             output.append('')
//         return "\n".join(output)



// def unittest(self, mhz=24):
//     print("\nTesting USB ping()")
//     self.ping(3)
    
//     print("\nTesting USB ep0Ping()")
//     self.ep0Ping()
    
//     print("\nTesting USB enumeration")
//     print("getString(0,100): %s" % repr(self._do.getString(0,100)))
    
//     print("\nTesting USB EP MAX_PACKET_SIZE handling (ep0Peek(0xf000, 100))")
//     print(repr(self.ep0Peek(0xf000, 100)))

//     print("\nTesting USB EP MAX_PACKET_SIZE handling (peek(0xf000, 300))")
//     print(repr(self.peek(0xf000, 400)))

//     print("\nTesting USB poke/peek")
//     data = "".join([correctbytes(c) for c in range(120)])
//     where = 0xf300
//     self.poke(where, data)
//     ndata = self.peek(where, len(data))
//     if ndata != data:
//         print(" *FAILED*\n '%s'\n '%s'" % (data.encode("hex"), ndata.encode("hex")))
//         raise Exception(" *FAILED*\n '%s'\n '%s'" % (data.encode("hex"), ndata.encode("hex")))
//     else:
//         print("  passed  '%s'" % (ndata.encode("hex")))


    pub fn has_bootloader(&self) -> Result<bool, libusb::Error> {
//    # in SFR I2SCLKF0 & I2SCLKF1
//         if(self.peek(0xDF46,1) == '\xF0' and self.peek(0xDF47,1) == '\x0D'):
//             output.append("Bootloader:          CC-Bootloader")
//         else:
//             output.append("Bootloader:          Not installed")
        Ok(false)
    }

    // pub fn peek(&self, addrL i16, u8 count=1) -> result<Vec<u8>, libusb::Error> {
    //     vec![00]
    // }

    // fn send(&self, SystemCommand cmd, AppMailbox mbx) -> Result<bool{

    // }


    // ######## APPLICATION API ########
    // def recv(self, app, cmd=None, wait=USB_RX_WAIT):
    //     '''
    //     high-level USB EP5 receive.  
    //     checks the mbox for app "app" and command "cmd" and returns the next one in the queue
    //     if any of this does not exist yet, wait for a RECV event until "wait" times out.
    //     RECV events are generated by the low-level recv thread "runEP5_recv()"
    //     '''
    //     startTime = time.time()
    //     self.recv_event.clear() # an event is only interesting if we've already failed to find our message

    //     while (time.time() - startTime)*1000 < wait:
    //         try:
    //             b = self.recv_mbox.get(app)
    //             if b:
    //                 if self._debug: print("Recv msg",app,b,cmd, file=sys.stderr)
    //                 if cmd is None:
    //                     keys = list(b.keys())
    //                     if len(keys):
    //                         cmd = list(b.keys())[-1] # just grab one.   no guarantees on the order

    //             if b is not None and cmd is not None:
    //                 q = b.get(cmd)
    //                 if self._debug: print("debug(recv) q='%s'"%repr(q), file=sys.stderr)

    //                 if q is not None and self.rsema.acquire(False):
    //                     if self._debug>3: print(("rsema.UNlocked", "rsema.locked")[self.rsema.locked()],2)
    //                     try:
    //                         resp, rt = q.pop(0)

    //                         self.rsema.release()
    //                         if self._debug>3: print(("rsema.UNlocked", "rsema.locked")[self.rsema.locked()],2)

    //                         # bring it on home...  this is the way out.
    //                         return resp[4:], rt

    //                     except IndexError:
    //                         pass

    //                     except AttributeError:
    //                         sys.excepthook(*sys.exc_info())
    //                         pass

    //                     self.rsema.release()

    //             self.recv_event.wait(old_div((wait - (time.time() - startTime)*1000),1000)) # wait on recv event, with timeout of remaining time
    //             self.recv_event.clear() # clear event, if it's set

    //         except KeyboardInterrupt:
    //             sys.excepthook(*sys.exc_info())
    //             break
    //         except:
    //             sys.excepthook(*sys.exc_info())

    //     raise ChipconUsbTimeoutException

    // def recvAll(self, app, cmd=None):
    //     retval = self.recv_mbox.get(app,None)
    //     if retval is not None:
    //         if cmd is not None:
    //             b = retval
    //             if self.rsema.acquire():
    //                 #if self._debug: print ("rsema.UNlocked", "rsema.locked")[self.rsema.locked()],3
    //                 try:
    //                     retval = b.get(cmd)
    //                     b[cmd]=[]
    //                     if len(retval):
    //                         retval = [ (d[4:],t) for d,t in retval ] 
    //                 except:
    //                     sys.excepthook(*sys.exc_info())
    //                 finally:
    //                     self.rsema.release()
    //                     #if self._debug: print ("rsema.UNlocked", "rsema.locked")[self.rsema.locked()],3
    //         else:
    //             if self.rsema.acquire():
    //                 #if self._debug: print ("rsema.UNlocked", "rsema.locked")[self.rsema.locked()],4
    //                 try:
    //                     self.recv_mbox[app]={}
    //                 finally:
    //                     self.rsema.release()
    //                     #if self._debug: print ("rsema.UNlocked", "rsema.locked")[self.rsema.locked()],4
    //         return retval

        // def send(self, app, cmd, buf, wait=USB_TX_WAIT):
        // msg = "%c%c%s%s"%(app,cmd, struct.pack("<H",len(buf)),buf)
        // self.xsema.acquire()
        // self.xmit_queue.append(msg)
        // self.xmit_event.set()
        // self.xsema.release()
        // if self._debug: print("Sent Msg",msg.encode("hex"))
        // return self.recv(app, cmd, wait)

        // def getPartNum(self):
        // try:
        //     r = self.send(APP_SYSTEM, SYS_CMD_PARTNUM, "", 10000)
        //     r,rt = r
        // except ChipconUsbTimeoutException as e:
        //     r = None
        //     print("SETUP Failed.",e)

        // return ord(r)

    //    def ping(self, count=10, buf="ABCDEFGHIJKLMNOPQRSTUVWXYZ", wait=DEFAULT_USB_TIMEOUT, silent=False):
    //     good=0
    //     bad=0
    //     start = time.time()
    //     for x in range(count):
    //         istart = time.time()
            
    //         try:
    //             r = self.send(APP_SYSTEM, SYS_CMD_PING, buf, wait)
    //             r,rt = r
    //             istop = time.time()
    //             if not silent:
    //                 print("PING: %d bytes transmitted, received: %s (%f seconds)"%(len(buf), repr(r), istop-istart))
    //         except ChipconUsbTimeoutException as e:
    //             r = None
    //             if not silent:
    //                 print("Ping Failed.",e)
    //         if r==None:
    //             bad+=1
    //         else:
    //             good+=1
    //     stop = time.time()
    //     return (good,bad,stop-start)

    pub fn ping(&self) -> Result<bool, libusb::Error> {
        let mut in_vec = Vec::<u8>::with_capacity(self.max_input_size as usize);
        let in_buf = unsafe { slice::from_raw_parts_mut((&mut in_vec[..]).as_mut_ptr(), in_vec.capacity()) };
        // TODO: packet builder
        let outvec = vec![AppMailbox::AppSystem as u8,
                          SystemCommand::Ping as u8,
                          0,
                          0,];
        match self.handle.write_bulk(self.out_endpoint_address, &outvec[..], self.timeout) {
            Ok(_) => (),
            Err(err) => {
                println!("nope: {}", err);
                return Err(err);
            },
        }
        match self.handle.read_bulk(self.in_endpoint_address, in_buf, self.timeout) {
            Ok(rlen) => {
                unsafe { in_vec.set_len(rlen) };
                assert_eq!(in_vec[0], 64);
                assert_eq!(in_vec[1], AppMailbox::AppSystem as u8);
                assert_eq!(in_vec[2], SystemCommand::Ping as u8);
                return Ok(true);
            },
            Err(err) => {
                println!("nope: {}", err);
                return Err(err);
            },
        }
    }

    pub fn bootloader(&self) -> Result<bool, libusb::Error> {
        let mut in_vec = Vec::<u8>::with_capacity(self.max_input_size as usize);
        let in_buf = unsafe { slice::from_raw_parts_mut((&mut in_vec[..]).as_mut_ptr(), in_vec.capacity()) };
        // TODO: packet builder
        let outvec = vec![AppMailbox::AppSystem as u8,
                          SystemCommand::Bootloader as u8,
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
                assert_eq!(in_vec[2], SystemCommand::Bootloader as u8);
                return Ok(true);
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

pub fn all_rfcats(context: &libusb::Context) -> Vec<RFCatDevice> {
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