#![no_std]
const AW9523_ADDRESS: u8 = 0x58;
pub enum DataFormat<'a> {
    /// Slice of unsigned bytes
    U8(&'a [u8]),
}

#[derive(Debug)]
pub enum Aw9523Error {
    NotSupported,
    InvalidArgument,
    ReadError,
    WriteError,
}

pub trait Aw9523ReadWrite {
    fn send_commands(&mut self, cmd: DataFormat<'_>) -> Result<(), Aw9523Error>;
}

pub struct Aw9523<I> {
    interface: I,
}

// https://github.com/m5stack/M5CoreS3/blob/main/src/AXP2101.cpp
impl<I> Aw9523<I>
where
    I: Aw9523ReadWrite,
{
    // Create a new AW9523 interface
    pub fn new(interface: I) -> Self {
        Self { interface }
    }

    // Initialize AW9523
    pub fn init(&mut self) -> Result<(), Aw9523Error> {
        let _ = self.interface.send_commands(DataFormat::U8(&[0x02, 0b00000101]));
        let _ = self.interface.send_commands(DataFormat::U8(&[0x03, 0b00000011]));
        let _ = self.interface.send_commands(DataFormat::U8(&[0x04, 0b00011000]));
        let _ = self.interface.send_commands(DataFormat::U8(&[0x05, 0b00001100]));
        let _ = self.interface.send_commands(DataFormat::U8(&[0x11, 0b00010000]));
        let _ = self.interface.send_commands(DataFormat::U8(&[0x13, 0b11111111]));

        Ok(())
    }
}

pub struct I2CInterface<I2C> {
    i2c: I2C,
    addr: u8,
}

impl<I2C> I2CInterface<I2C>
where
    I2C: embedded_hal::blocking::i2c::Write,
{
    /// Create new I2C interface for communication with a display driver
    pub fn new(i2c: I2C, addr: u8) -> Self {
        Self {
            i2c,
            addr,
        }
    }

    /// Consume the display interface and return
    /// the underlying peripherial driver
    pub fn release(self) -> I2C {
        self.i2c
    }
}

// Implement Aw9523ReadWrite for I2CInterface
impl<I> Aw9523ReadWrite for I2CInterface<I>
where
    I: embedded_hal::blocking::i2c::Write + embedded_hal::blocking::i2c::WriteRead,
{
    // Send commands over I2C to Aw9523
    fn send_commands(&mut self, cmd: DataFormat<'_>) -> Result<(), Aw9523Error> {
        let mut data_buf = [0];

        match cmd {
            DataFormat::U8(data) => {
                self.i2c
                    .write_read(self.addr, &[data[0]], &mut data_buf)
                    .map_err(|_| Aw9523Error::WriteError)?;
                self.i2c
                    .write(self.addr, data)
                    .map_err(|_| Aw9523Error::WriteError)
            }
        }
    }

}


#[derive(Debug, Copy, Clone)]
pub struct I2CGpioExpanderInterface(());

impl I2CGpioExpanderInterface {
    pub fn new<I>(i2c: I) -> I2CInterface<I>
    where
        I: embedded_hal::blocking::i2c::Write,
    {
        Self::new_custom_address(i2c, AW9523_ADDRESS)
    }

    /// Create a new I2C interface with a custom address.
    pub fn new_custom_address<I>(i2c: I, address: u8) -> I2CInterface<I>
    where
        I: embedded_hal::blocking::i2c::Write,
    {
        I2CInterface::new(i2c, address)
    }
}
