#![allow(unused)]
use std::{
    any::{Any, TypeId}, fmt::{Debug, Display}, fs::File, io::{stdin, stdout, Write}, process, str::FromStr, time::Duration
};

use clap::Parser;
use libusb::{
    Direction, EndpointDescriptor, InterfaceDescriptor, SyncType, TransferType, UsageType, Version,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub enum Mode {
    Info,
    Read,
    Write,
}

impl From<&str> for Mode {
    fn from(s: &str) -> Self {
        match s {
            "info" => Self::Info,
            "read" => Self::Read,
            "write" => Self::Write,
            _ => {
                println!("[Error] {} is not a valid mode", s);
                process::exit(1);
            }
        }
    }
}

#[derive(Parser, Debug)]
#[command(version)]
pub struct AppArgs {
    /// Select the usb interface get this using info mode.
    #[arg(short)]
    interface: Option<u8>,

    /// Select a endpoint get this using info mode.
    #[arg(short)]
    endpoint: Option<u8>,

    /// Usb configuration get this using info mode.
    #[arg(short)]
    configuration: Option<u8>,

    /// Device name: you can get this from lsusb.
    #[arg(short)]
    device: Option<String>,

    /// Operation mode: [info, read].
    #[arg(short)]
    mode: Mode,
}

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Clone)]
pub enum ClassCode {
    Audio,
    CommunicationAndCdcDescriptors,
    HumanInterfaceDevice,
    Physical,
    Image,
    Printer,
    MassStorage,
    Hub,
    CdcData,
    SmartCard,
    ContentSecurity,
    Video,
    PersonalHealthcare,
    AudioVideoDevices,
    BillboardDeviceClass,
    USBTypeCBridgeClass,
    USBBulkDisplayProtocolDeviceClass,
    MCTPOverUSBProtocolDeviceClass,
    I3CDeviceClass,
    DiagnosticDevice,
    WirelessController,
    Miscellaneous,
    ApplicationSpecific,
    VenderSpecific,
    Unknown,
}

impl Debug for ClassCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClassCode::Audio => f.write_str("Audio"),
            ClassCode::CommunicationAndCdcDescriptors => {
                f.write_str("Communications and CDC Control")
            }
            ClassCode::HumanInterfaceDevice => f.write_str("HID (Human Interface Device)"),
            ClassCode::Physical => f.write_str("Physical"),
            ClassCode::Image => f.write_str("Image"),
            ClassCode::Printer => f.write_str("Printer"),
            ClassCode::MassStorage => f.write_str("Mass Storage Device"),
            ClassCode::Hub => f.write_str("Hub"),
            ClassCode::CdcData => f.write_str("CDC-Data"),
            ClassCode::SmartCard => f.write_str("Smart Card"),
            ClassCode::ContentSecurity => f.write_str("Content Security"),
            ClassCode::Video => f.write_str("Video"),
            ClassCode::PersonalHealthcare => f.write_str("Personal Health Care"),
            ClassCode::AudioVideoDevices => f.write_str("Audio/Video Devices"),
            ClassCode::BillboardDeviceClass => f.write_str("Billboard Device Class"),
            ClassCode::USBTypeCBridgeClass => f.write_str("USB Type C Bridge Class"),
            ClassCode::USBBulkDisplayProtocolDeviceClass => {
                f.write_str("USB Bulk Display Protocol Class")
            }
            ClassCode::MCTPOverUSBProtocolDeviceClass => {
                f.write_str("MCTP Over USB Protocol Device Class")
            }
            ClassCode::I3CDeviceClass => f.write_str("I3C Device Class"),
            ClassCode::DiagnosticDevice => f.write_str("Diagnostic Device"),
            ClassCode::WirelessController => f.write_str("Wireless Controller"),
            ClassCode::Miscellaneous => f.write_str("Miscellaneous"),
            ClassCode::ApplicationSpecific => f.write_str("Application Specific"),
            ClassCode::VenderSpecific => f.write_str("Vender Specific"),
            ClassCode::Unknown => f.write_str("Unknown Device Class"),
        }
    }
}

impl ClassCode {
    fn from_u8(num: u8) -> Self {
        match num {
            0x01 => Self::Audio,
            0x02 => Self::CommunicationAndCdcDescriptors,
            0x03 => Self::HumanInterfaceDevice,
            0x05 => Self::Physical,
            0x06 => Self::Image,
            0x07 => Self::Printer,
            0x08 => Self::MassStorage,
            0x09 => Self::Hub,
            0x0A => Self::CdcData,
            0x0B => Self::SmartCard,
            0x0D => Self::ContentSecurity,
            0x0E => Self::Video,
            0x0F => Self::PersonalHealthcare,
            0x10 => Self::AudioVideoDevices,
            0x11 => Self::BillboardDeviceClass,
            0x12 => Self::USBTypeCBridgeClass,
            0x13 => Self::USBBulkDisplayProtocolDeviceClass,
            0x14 => Self::MCTPOverUSBProtocolDeviceClass,
            0x3C => Self::I3CDeviceClass,
            0xDC => Self::DiagnosticDevice,
            0xE0 => Self::WirelessController,
            0xEF => Self::Miscellaneous,
            0xFE => Self::ApplicationSpecific,
            0xFF => Self::VenderSpecific,
            _ => Self::Unknown,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DeviceInfo {
    pub vendor_id: String,
    pub product_id: String,
    pub class_code: ClassCode,
    pub subclass_code: u8,
    pub num_configurations: u8,
    // pub usb_version: Version,
    pub protocol_code: u8,
    pub max_packet_size: u8,
    // pub product_string_index: Option<u8>,
    // pub manufacturer_string_index: Option<u8>,
    // pub serial_number_string_index: Option<u8>,
    // pub type_id: TypeId,
    pub configurations: Vec<ConfigDescriptor>,
}

impl DeviceInfo {
    fn get_id(&self) -> String {
        format!("{}:{}", self.vendor_id, self.product_id)
    }
}

#[derive(Debug, Clone)]
pub struct Interfaces {
    pub number: u8,
    pub num_endpoints: u8,
    pub interface_number: u8,
    pub class_code: ClassCode,
    pub subclass_code: u8,
    pub description_string_index: Option<u8>,
    pub endpoints: Vec<Endpoint>,
}

impl Interfaces {
    pub fn new(i: &InterfaceDescriptor) -> Self {
        Self {
            number: i.interface_number(),
            num_endpoints: i.num_endpoints(),
            interface_number: i.interface_number(),
            class_code: ClassCode::from_u8(i.class_code()),
            subclass_code: i.sub_class_code(),
            description_string_index: i.description_string_index(),
            endpoints: i
                .endpoint_descriptors()
                .into_iter()
                .map(|endpoint| Endpoint::new(&endpoint))
                .collect::<Vec<_>>(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConfigDescriptor {
    pub num_interfaces: u8,
    pub max_power: u16,
    pub self_powered: bool,
    pub number: u8,
    pub remote_wakeup: bool,
    pub interfaces: Vec<Interfaces>,
}

#[derive(Debug, Clone)]
pub struct Endpoint {
    pub max_packet_size: u16,
    pub endpoint_number: u8,
    pub interval: u8,
    pub transfer_type: TransferType,
    pub sync_type: SyncType,
    pub address: u8,
    pub direction: Direction,
    pub usage_type: UsageType,
}
impl Endpoint {
    pub fn new(ep: &EndpointDescriptor) -> Self {
        Self {
            max_packet_size: ep.max_packet_size(),
            endpoint_number: ep.number(),
            interval: ep.interval(),
            transfer_type: ep.transfer_type(),
            sync_type: ep.sync_type(),
            direction: ep.direction(),
            usage_type: ep.usage_type(),
            address: ep.address(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct UsbDevices {
    devices: Vec<DeviceInfo>,
}

impl UsbDevices {
    pub fn new() -> Result<Self> {
        let ctx = libusb::Context::new()?;

        let devices = ctx.devices()?;

        let mut usb_devices = Self {
            devices: Vec::new(),
        };

        for i in devices.iter() {
            let d = i.device_descriptor()?;

            let mut config = DeviceInfo {
                vendor_id: format!("{:04x}", d.vendor_id()),
                product_id: format!("{:04x}", d.product_id()),
                class_code: ClassCode::from_u8(d.class_code()),
                subclass_code: d.sub_class_code(),
                num_configurations: d.num_configurations(),
                protocol_code: d.protocol_code(),
                max_packet_size: d.max_packet_size(),
                configurations: Vec::new(),
            };

            for c in 0..d.num_configurations() {
                let c = i.config_descriptor(c)?;

                let mut config_descriptor = ConfigDescriptor {
                    number: c.number(),
                    max_power: c.max_power(),
                    self_powered: c.self_powered(),
                    remote_wakeup: c.remote_wakeup(),
                    num_interfaces: c.num_interfaces(),
                    interfaces: Vec::new(),
                };

                for interface in c.interfaces() {
                    for interface_descriptor in interface.descriptors().into_iter() {
                        let interface_s = Interfaces::new(&interface_descriptor);

                        config_descriptor.interfaces.push(interface_s);
                    }
                }
                config.configurations.push(config_descriptor);
            }

            usb_devices.devices.push(config);
        }

        Ok(usb_devices)
    }
    pub fn get_by_id(&self, id: &str) -> Option<DeviceInfo> {
        self.devices
            .iter()
            .find(|e| e.get_id() == id)
            .map(|d| d.clone())
    }

    pub fn print_info(&self) {
        println!("{:#?}", &self.devices);
    }
}

pub fn get_device_info(dev: &Option<String>) -> Result<()> {
    let dev_info = UsbDevices::new()?;
    if let Some(dev) = dev {
        if let Some(info) = dev_info.get_by_id(&dev) {
            dbg!(info);
        } else {
            println!("[device {} not found.]", dev);
        }
    } else {
        dev_info.print_info();
    }
    Ok(())
}

#[derive(Debug,Serialize,Deserialize)]
struct Keymap {
    key_name: String,
    map: [u8; 8],
}

fn store_keymap() {}

pub fn write_to_device(args: &AppArgs) {
    let endpoint = args.endpoint.unwrap_or(129);
    let interface = args.interface.unwrap_or(1);
    let config = args.configuration.unwrap();
    let device = args.device.as_ref().unwrap();

    let mut ctx = libusb::Context::new().unwrap();

    let dev = ctx.devices().unwrap().iter().find(|d| {
        let descriptor = d.device_descriptor().unwrap();
        let name = format!(
            "{:04x}:{:04x}",
            descriptor.vendor_id(),
            descriptor.product_id()
        );
        if name == *device {
            return true;
        } else {
            return false;
        }
    });

    let mut handler = dev.unwrap().open().unwrap();

    handler.set_active_configuration(config);
    handler.detach_kernel_driver(interface);
    handler.claim_interface(interface);

    let mut keymaps = Vec::new();

    'outer: loop {
        loop {
            let mut buf = [0u8; 8];
            let size = handler
                .read_interrupt(endpoint, &mut buf, Duration::from_millis(100))
                .unwrap_or(0);
            if size == 0 {
                break;
            }
        }

        print!("Input a name: ");
        stdout().flush();

        let mut s = String::new();
        stdin().read_line(&mut s).unwrap();

        println!("Press a button on your keyboard: ");

        let mut buf = [0u8; 8];
        handler.read_interrupt(endpoint, &mut buf, Duration::default());

        let keyname = s.strip_suffix("\n").unwrap();
        println!("Key {} => {:?}", keyname, buf);
        println!("---------------------------------------------------------");

        keymaps.push(Keymap {
            key_name: keyname.to_string(),
            map: buf,
        });

        'options: loop {
            print!("Q: Quit | N: Next => ");
            stdout().flush();
            
            let mut s = String::new();
            stdin().read_line(&mut s).unwrap();
            let s = s.strip_suffix("\n").unwrap();

            println!("---------------------------------------------------------");

            match s {
                "Q" | "q" => break 'outer,
                "N" | "n" => break 'options,
                _ => {
                    println!("{} is not a valid option", s)
                }
            }
        }
    }
    let map = serde_json::to_string(&keymaps).unwrap();
    let mut file = File::create("config.json").unwrap();
    file.write_all(map.as_bytes());
}

fn main() -> Result<()> {
    let args = AppArgs::parse();


    match args.mode {
        Mode::Info => {
            get_device_info(&args.device);
        }
        Mode::Read => {
            write_to_device(&args);
        }
        Mode::Write => todo!(),
    };

    Ok(())
}
