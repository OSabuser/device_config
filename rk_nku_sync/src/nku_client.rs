use crate::serial_config::SerialPortConfig;
use config_lib::device_config::DeviceConfig;
use log::{debug, info};
use protocol_lib::client::HostClient;

const BOARD_RESPONSE_TIMEOUT_MS: std::time::Duration = std::time::Duration::from_millis(5000);
const SERIAL_PORT_CONFIG_PATH: &str = "rk_nku_configs/rk3399_scheme.toml";
const NKU_DEVICE_CONFIG_PATH: &str = "rk_nku_configs/nku_scheme.toml";

#[derive(Clone)]
/// Режимы стриминга данных от устройства
pub enum StreamingMode {
    /// Без стриминга
    __SilentMode = 0,
    /// Отправка данных, в случае изменения их состояния (и наличия)
    OnChangeMode = 1,
    /// Отправка данных периодически, с заданным в настройках (periodicity) периодом
    __PeriodicMode = 2,
    /// Отправка данных по требованию
    __OnDemandMode = 3,
}

pub struct NkuClient {
    nku_client: HostClient,
    nku_config: DeviceConfig,
}

impl NkuClient {
    pub fn new() -> Result<Self, String> {
        // Чтение параметров последовательного порта
        let serial_config = SerialPortConfig::new(SERIAL_PORT_CONFIG_PATH)?;

        let nku_config = DeviceConfig::create_parameter_list(NKU_DEVICE_CONFIG_PATH)?;
        debug!("Parameters list: {:#?}", nku_config.get_parameters_names());

        let port_name = serial_config.get_serial_name();
        let baudrate = serial_config.get_serial_baudrate();

        debug!("Serial config: {port_name}, {baudrate}");

        let nku_client = HostClient::connect(&port_name, baudrate, BOARD_RESPONSE_TIMEOUT_MS)?;

        info!("Connection with IMv has been established!");

        Ok(Self {
            nku_client,
            nku_config,
        })
    }

    /// ### Запрос начала стриминга данных со станции управления
    pub fn start_elevator_data_streaming(&mut self, mode: StreamingMode) -> Result<String, String> {
        let response_from_mcu = self
            .nku_client
            .send_request(format!("set mode {}", mode.clone() as u8).as_str())?;
        if !response_from_mcu.contains(format!("mode: {}", mode as u8).as_str()) {
            return Err("START STREAMING> invalid response".into());
        }
        Ok(response_from_mcu)
    }

    /// ### Запрос сохраненных в устройстве настроек
    pub fn pull_parameters_from_device(&mut self) -> Result<(), String> {
        let parameters_list = self.nku_config.get_parameters_names()?;

        for parameter in parameters_list {
            let request_string = format!("get {parameter}");
            debug!("PULL> sending request: {request_string}");
            let response_from_mcu = self.nku_client.send_request(&request_string)?;
            debug!("PULL> response from MCU: {response_from_mcu}");

            let parameter_value =
                NkuClient::extract_parameter_value(&parameter, response_from_mcu)?;
            self.nku_config
                .set_parameter_value_using_index(&parameter, parameter_value)?;
        }
        self.nku_config.save_parameters_values()?;
        Ok(())
    }

    /// Извлечение значения параметра из отклика от MCU
    fn extract_parameter_value(
        parameter_name: &str,
        response_from_mcu: String,
    ) -> Result<u8, String> {
        // Ожидается ответ в формате "parameter_name:value"
        let tokens = response_from_mcu.split(':').collect::<Vec<&str>>().to_vec();

        if tokens.len() != 2 {
            return Err(format!(
                "Failed to parse response from MCU: {response_from_mcu}"
            ));
        }

        if tokens[0] != parameter_name {
            return Err(format!(
                "Failed to parse response from MCU: {response_from_mcu}"
            ));
        }

        tokens[1].trim().parse::<u8>().map_err(|e| e.to_string())
    }

    /// Отправка новых настроек на устройство для последующего сохранения
    pub fn push_parameters_to_device(&mut self) -> Result<(), String> {
        let parameters_list = self.nku_config.get_parameters_names()?;

        for parameter in parameters_list {
            let parameter_value = self
                .nku_config
                .get_parameter_index_using_value(&parameter)?;
            let request_string = format!("set {parameter} {parameter_value}");
            debug!("PUSH> sending request: {request_string}");
            let response_from_mcu = self.nku_client.send_request(&request_string)?;

            debug!("PUSH> response from MCU: {response_from_mcu}");
            if !response_from_mcu.contains(format!("{parameter}: {parameter_value}").as_str()) {
                return Err("PUSH> invalid response".into());
            }
        }

        Ok(())
    }
}
