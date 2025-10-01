use config_lib::device_config::DeviceConfig;
use log::debug;

pub struct SerialPortConfig {
    serial_name: String,
    serial_baudrate: u32,
}

impl SerialPortConfig {
    /// Чтение параметров последовательного порта из файла-схемы TOML
    pub fn new(path_to_scheme: &str) -> Result<Self, String> {
        let nku_serial_parameters = DeviceConfig::create_parameter_list(path_to_scheme)?;

        let serial_name = nku_serial_parameters.get_parameter_value("device")?;
        let serial_baudrate = nku_serial_parameters
            .get_parameter_value("baudrate")?
            .parse::<u32>()
            .map_err(|e: std::num::ParseIntError| e.to_string())?;

        debug!("serial port_name: {}", serial_name);
        debug!("serial baudrate: {}", serial_baudrate);

        Ok(SerialPortConfig {
            serial_name,
            serial_baudrate,
        })
    }

    /// Чтение имени последовательного порта из конфига
    pub fn get_serial_name(&self) -> String {
        self.serial_name.clone()
    }

    /// Чтение скорости последовательного порта из конфига
    pub fn get_serial_baudrate(&self) -> u32 {
        self.serial_baudrate
    }
}
