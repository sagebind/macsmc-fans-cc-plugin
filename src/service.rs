use crate::{
    SERVICE_ID, VERSION,
    device_service::v1::{
        CustomFunctionOneRequest, CustomFunctionOneResponse, EnableManualFanControlRequest,
        EnableManualFanControlResponse, FixedDutyRequest, FixedDutyResponse, HealthRequest,
        HealthResponse, InitializeDeviceRequest, InitializeDeviceResponse, LcdRequest, LcdResponse,
        LightingRequest, LightingResponse, ListDevicesRequest, ListDevicesResponse,
        ResetChannelRequest, ResetChannelResponse, ShutdownRequest, ShutdownResponse,
        SpeedProfileRequest, SpeedProfileResponse, StatusRequest, StatusResponse,
        device_service_server::DeviceService, health_response,
    },
    models::{
        self,
        v1::{
            ChannelInfo, Device, DeviceInfo, SpeedOptions, channel_info::Options, status::FanSpeed,
        },
    },
    tuxedo_io::{Fan, TuxedoIo},
};
use std::{collections::HashMap, io, sync::Arc};
use sysinfo::Product;
use tokio::{sync::Mutex, task::spawn_blocking, time::Instant};
use tonic::{Request, Response, Status};

const DEVICE_ID: &str = "tuxedo";
const DEFAULT_DEVICE_NAME: &str = "TUXEDO InfinityBook Gen10";
const FAN_1_CHANNEL_ID: &str = "fan1";
const FAN_2_CHANNEL_ID: &str = "fan2";

pub struct TuxedoService {
    start_time: Instant,
    tuxedo_io: Arc<Mutex<Option<TuxedoIo>>>,
}

impl TuxedoService {
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            tuxedo_io: Arc::new(Mutex::new(None)),
        }
    }

    async fn with_io<T: Send + 'static>(
        &self,
        f: impl Send + FnOnce(&mut Option<TuxedoIo>) -> Result<T, Status> + 'static,
    ) -> Result<T, Status> {
        let arc = self.tuxedo_io.clone();

        spawn_blocking(move || {
            let mut tuxedo_io = arc.blocking_lock();
            f(&mut *tuxedo_io)
        })
        .await
        .map_err(|e| Status::from_error(Box::new(e)))
        .flatten()
    }

    async fn with_io_initialized<T: Send + 'static>(
        &self,
        f: impl Send + FnOnce(&TuxedoIo) -> Result<T, Status> + 'static,
    ) -> Result<T, Status> {
        self.with_io(move |tuxedo_io| {
            let tuxedo_io = match tuxedo_io.as_mut() {
                Some(io) => io,
                None => tuxedo_io.insert(TuxedoIo::open()?),
            };

            f(tuxedo_io)
        })
        .await
    }
}

#[tonic::async_trait]
impl DeviceService for TuxedoService {
    async fn health(
        &self,
        _request: Request<HealthRequest>,
    ) -> Result<Response<HealthResponse>, Status> {
        let reply = HealthResponse {
            name: SERVICE_ID.to_string(),
            version: VERSION.to_string(),
            status: health_response::Status::Ok.into(),
            uptime_seconds: self.start_time.elapsed().as_secs(),
        };
        Ok(Response::new(reply))
    }

    async fn list_devices(
        &self,
        _request: Request<ListDevicesRequest>,
    ) -> Result<Response<ListDevicesResponse>, Status> {
        self.with_io_initialized(|tuxedo_io| {
            let device = get_device(tuxedo_io)?;

            Ok(Response::new(ListDevicesResponse {
                devices: vec![device],
            }))
        })
        .await
    }

    async fn initialize_device(
        &self,
        _request: Request<InitializeDeviceRequest>,
    ) -> Result<Response<InitializeDeviceResponse>, Status> {
        self.with_io_initialized(|_| {
            // Nothing else to do, enter will ensure a connection is established.

            Ok(Response::new(InitializeDeviceResponse {}))
        })
        .await
    }

    async fn shutdown(
        &self,
        _request: Request<ShutdownRequest>,
    ) -> Result<Response<ShutdownResponse>, Status> {
        self.with_io(|tuxedo_io| {
            // Reset the fans to auto before exiting, or they may be stuck off
            // which could cause overheating.
            if let Some(tuxedo_io) = tuxedo_io.take() {
                tuxedo_io.set_fans_auto()?;

                // Disconnect the driver handle.
                drop(tuxedo_io);
            }

            Ok(Response::new(ShutdownResponse {}))
        })
        .await
    }

    async fn status(
        &self,
        _request: Request<StatusRequest>,
    ) -> Result<Response<StatusResponse>, Status> {
        self.with_io_initialized(|tuxedo_io| {
            Ok(Response::new(StatusResponse {
                status: vec![
                    models::v1::Status {
                        id: FAN_1_CHANNEL_ID.into(),
                        metric: Some(models::v1::status::Metric::Speed(FanSpeed {
                            duty: Some(tuxedo_io.get_fan_speed(Fan::Fan1)? as f64),
                            rpm: None,
                        })),
                    },
                    models::v1::Status {
                        id: FAN_2_CHANNEL_ID.into(),
                        metric: Some(models::v1::status::Metric::Speed(FanSpeed {
                            duty: Some(tuxedo_io.get_fan_speed(Fan::Fan2)? as f64),
                            rpm: None,
                        })),
                    },
                ],
            }))
        })
        .await
    }

    async fn reset_channel(
        &self,
        _request: Request<ResetChannelRequest>,
    ) -> Result<Response<ResetChannelResponse>, Status> {
        self.with_io_initialized(|tuxedo_io| {
            tuxedo_io.set_fans_auto()?;

            Ok(Response::new(ResetChannelResponse {}))
        })
        .await
    }

    async fn enable_manual_fan_control(
        &self,
        _request: Request<EnableManualFanControlRequest>,
    ) -> Result<Response<EnableManualFanControlResponse>, Status> {
        // Nothing to do, will automatically activate manual control when setting
        // a speed.
        Ok(Response::new(EnableManualFanControlResponse {}))
    }

    async fn fixed_duty(
        &self,
        request: Request<FixedDutyRequest>,
    ) -> Result<Response<FixedDutyResponse>, Status> {
        self.with_io_initialized(move |tuxedo_io| {
            let fan = if request.get_ref().channel_id == FAN_1_CHANNEL_ID {
                Fan::Fan1
            } else if request.get_ref().channel_id == FAN_2_CHANNEL_ID {
                Fan::Fan2
            } else {
                return Err(Status::invalid_argument("Unknown channel ID"));
            };

            tuxedo_io.set_fan_speed(fan, request.get_ref().duty as u8)?;

            Ok(Response::new(FixedDutyResponse {}))
        })
        .await
    }

    async fn speed_profile(
        &self,
        _request: Request<SpeedProfileRequest>,
    ) -> Result<Response<SpeedProfileResponse>, Status> {
        // TODO: Apply a speed profile to the device channel
        Err(Status::unimplemented("No Firmware Profiles"))
    }

    async fn lighting(
        &self,
        _request: Request<LightingRequest>,
    ) -> Result<Response<LightingResponse>, Status> {
        Err(Status::unimplemented("No Lighting Channels"))
    }

    async fn lcd(&self, _request: Request<LcdRequest>) -> Result<Response<LcdResponse>, Status> {
        Err(Status::unimplemented("No LCD Channels"))
    }

    async fn custom_function_one(
        &self,
        _request: Request<CustomFunctionOneRequest>,
    ) -> Result<Response<CustomFunctionOneResponse>, Status> {
        Err(Status::unimplemented("No Custom Function"))
    }
}

impl Drop for TuxedoService {
    fn drop(&mut self) {
        // Ensure that fan control is always relinquished to the firmware when we
        // stop controlling it, even if a proper shutdown sequence did not occur.
        if let Some(tuxedo_io) = self.tuxedo_io.blocking_lock().take() {
            let _ = tuxedo_io.set_fans_auto();
        }
    }
}

fn get_device(tuxedo_io: &TuxedoIo) -> io::Result<Device> {
    let min_duty = tuxedo_io.get_fan_min_speed()?.into();
    let max_duty = 100;

    let mut channels = HashMap::new();

    channels.insert(
        FAN_1_CHANNEL_ID.into(),
        ChannelInfo {
            label: Some("Fan 1".into()),
            options: Some(Options::SpeedOptions(SpeedOptions {
                min_duty,
                max_duty,
                fixed_enabled: true,
                ..Default::default()
            })),
        },
    );

    channels.insert(
        FAN_2_CHANNEL_ID.into(),
        ChannelInfo {
            label: Some("Fan 2".into()),
            options: Some(Options::SpeedOptions(SpeedOptions {
                min_duty,
                max_duty,
                fixed_enabled: true,
                ..Default::default()
            })),
        },
    );

    Ok(Device {
        id: DEVICE_ID.into(),
        name: Product::name().unwrap_or_else(|| DEFAULT_DEVICE_NAME.into()),
        uid_info: None,
        info: Some(DeviceInfo {
            channels,
            ..Default::default()
        }),
    })
}
