use esp_idf_svc::{
    eventloop::EspSystemEventLoop, hal::modem::Modem, nvs::EspDefaultNvsPartition, sntp::{EspSntp, SyncStatus}, sys::EspError, wifi::{self, BlockingWifi, EspWifi}
};

use crate::feedback::FeedbackSystem;

const SSID: &str = env!("WIFI_SSID");
const PASSWORD: &str = env!("WIFI_PASS");

/// Establishes network connection to wifi with configuration provided by environment variables.
/// Wifi is needed for connecting to SNTP server and getting current time.
pub fn wifi_connect(modem: Modem, sys_loop: EspSystemEventLoop, nvs: EspDefaultNvsPartition, feedback: &mut impl FeedbackSystem) -> Result<esp_idf_svc::wifi::EspWifi<'static>, EspError> {
    let mut esp_wifi = EspWifi::new(modem, sys_loop.clone(), Some(nvs))?;
    let mut wifi = BlockingWifi::wrap(&mut esp_wifi, sys_loop)?;

    wifi.set_configuration(&wifi::Configuration::Client(wifi::ClientConfiguration {
        ssid: SSID.try_into().unwrap(),
        password: PASSWORD.try_into().unwrap(),
        ..Default::default()
    }))?;

    feedback.write_msg("Starting wifi...");
    wifi.start()?;
    log::info!("Wifi started");

    feedback.write_msg(SSID);

    feedback.write_msg("Connecting to wifi...");
    wifi.connect()?;
    log::info!("Wifi connected");

    feedback.write_msg("Waiting for netif...");
    wifi.wait_netif_up()?;
    log::info!("Wifi netif up");

    Ok(esp_wifi)
}

// Connects to SNTP server for current time. Needs wifi connection first.
pub fn sntp_connect(feedback: &mut impl FeedbackSystem) -> Result<EspSntp<'static>, EspError> {
    feedback.write_msg("Initializing SNTP...");
    let sntp = EspSntp::new_default()?;
    while sntp.get_sync_status() == SyncStatus::InProgress { }
    log::info!("SNTP initialized");
    Ok(sntp)
}