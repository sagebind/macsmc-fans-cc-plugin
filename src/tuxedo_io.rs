use crate::sys::{UW_MAX_FAN_SPEED, ioctl};
use std::{
    fs::OpenOptions,
    io::{Error, ErrorKind, Result},
    os::fd::{AsRawFd, OwnedFd},
};

/// Safe wrapper around the Tuxedo driver IOCTL interface.
///
/// Note that this assumes Gen10 Uniwill hardware, since that's my device. I have
/// not implemented support for anything else.
pub struct TuxedoIo(OwnedFd);

#[derive(Debug, Clone, Copy)]
pub enum Fan {
    Fan1,
    Fan2,
}

impl TuxedoIo {
    pub fn open() -> Result<Self> {
        let fd: OwnedFd = OpenOptions::new()
            .read(true)
            .write(true)
            .open("/dev/tuxedo_io")?
            .into();

        let mut code = 0;

        unsafe {
            ioctl::uw_hwcheck(fd.as_raw_fd(), &mut code)?;
        }

        if code == 1 {
            Ok(TuxedoIo(fd))
        } else {
            Err(Error::new(ErrorKind::Other, "hardware check failed"))
        }
    }

    /// Get the minimum recommended fan speed for all fans, as a percentage.
    pub fn get_fan_min_speed(&self) -> Result<u8> {
        let mut value = 0;

        unsafe {
            ioctl::r_uw_fans_min_speed(self.0.as_raw_fd(), &mut value)?;
        }

        Ok(speed_to_percentage(value))
    }

    /// Get the current speed of a fan as a percentage.
    pub fn get_fan_speed(&self, fan: Fan) -> Result<u8> {
        let mut value = 0;

        unsafe {
            match fan {
                Fan::Fan1 => ioctl::r_uw_fanspeed(self.0.as_raw_fd(), &mut value)?,
                Fan::Fan2 => ioctl::r_uw_fanspeed2(self.0.as_raw_fd(), &mut value)?,
            };
        }

        Ok(speed_to_percentage(value))
    }

    /// Set the desired speed of a fan as a percentage.
    ///
    /// This function is blocking. The driver will not return until the desired
    /// speed is reached.
    pub fn set_fan_speed(&self, fan: Fan, percentage: u8) -> Result<()> {
        let value = percentage_to_speed(percentage).into();

        unsafe {
            match fan {
                Fan::Fan1 => ioctl::w_uw_fanspeed(self.0.as_raw_fd(), &value)?,
                Fan::Fan2 => ioctl::w_uw_fanspeed2(self.0.as_raw_fd(), &value)?,
            };
        }

        Ok(())
    }

    /// Set all fans to default mode (controlled by firmware).
    pub fn set_fans_auto(&self) -> Result<()> {
        unsafe {
            ioctl::w_uw_fanauto(self.0.as_raw_fd())?;
        }

        Ok(())
    }
}

fn speed_to_percentage(speed: i32) -> u8 {
    (speed as f32 / UW_MAX_FAN_SPEED as f32 * 100f32) as u8
}

fn percentage_to_speed(percentage: u8) -> u8 {
    (UW_MAX_FAN_SPEED as f32 * percentage as f32 / 100f32) as u8
}
