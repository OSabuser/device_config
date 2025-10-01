use log::debug;
use std::collections::HashMap;
use toml_edit::value;

use crate::toml_parser::*;

/// Структура, содержащая набор параметров Parameter
#[derive(Debug, Clone)]
pub struct DeviceConfig {
    scheme: TomlScheme,
    /// Набор параметров
    /// key - имя параметра, value - структура, описывающая параметр
    parameters: HashMap<String, Parameter>,
}

impl DeviceConfig {
    /// ## Заполнение струтуры DeviceConfig
    /// * `path_to_scheme` - путь к файлу-cхеме
    pub fn create_parameter_list(path_to_scheme: &str) -> Result<DeviceConfig, String> {
        let toml_config = TomlScheme::new(path_to_scheme).map_err(|e| e.to_string())?;

        let parameter_list = toml_config
            .get_list_of_parameters()
            .map_err(|e| e.to_string())?;

        debug!("Parameter list: {:#?}", parameter_list);
        let mut parameter_map = HashMap::new();

        for parameter in parameter_list {
            let device_parameter = Parameter {
                description: toml_config.get_string_value(&parameter, "name")?,
                value: toml_config.get_string_value(&parameter, "current")?,
                possible_values: toml_config.get_array_value(&parameter, "possible_values")?,
            };
            parameter_map.insert(parameter, device_parameter);
        }

        let device_config = DeviceConfig {
            scheme: toml_config,
            parameters: parameter_map,
        };

        Ok(device_config)
    }

    /// # Сохранение текущих значений параметров в TOML-файл self.schema_path
    pub fn save_parameters_values(&self) -> Result<(), String> {
        let mut toml_config =
            TomlScheme::new(&self.scheme.get_path_to_scheme_file()).map_err(|e| e.to_string())?;
        for (parameter_name, parameter_object) in &self.parameters {
            toml_config.set_parameter_value(
                parameter_name,
                "current",
                value(parameter_object.get_value()),
            )?;
        }
        Ok(())
    }

    /// # Получение списка ключей - имен параметров
    pub fn get_parameters_names(&self) -> Result<Vec<String>, String> {
        if self.parameters.is_empty() {
            return Err("Parameters list is empty".to_string());
        }

        return Ok(self.parameters.keys().map(|key| key.clone()).collect());
    }

    /// # Получение описания параметра соответствующего key
    pub fn get_parameter_description(&self, key: &str) -> Result<String, String> {
        match self.parameters.get(key) {
            Some(parameter) => Ok(parameter.get_description()),
            None => Err(format!("Parameter {} not found", key)),
        }
    }

    /// # Получение списка возможных значений параметра соответствующего key
    pub fn get_parameter_possible_values(&self, key: &str) -> Result<Vec<String>, String> {
        match self.parameters.get(key) {
            Some(parameter) => Ok(parameter.get_possible_values()),
            None => Err(format!("Parameter {} not found", key)),
        }
    }

    /// # Получение значения параметра соответствующего key
    pub fn get_parameter_value(&self, key: &str) -> Result<String, String> {
        match self.parameters.get(key) {
            Some(parameter) => Ok(parameter.get_value()),
            None => Err(format!("Parameter {} not found", key)),
        }
    }

    /// # Установка значения параметра соответствующего key
    pub fn set_parameter_value(&mut self, key: &str, value: String) -> Result<(), String> {
        match self.parameters.get_mut(key) {
            Some(parameter) => parameter.set_value(value),
            None => Err(format!("Parameter {} not found", key)),
        }
    }

    /// ## Получение числового индекса соответствующего текущему значению параметра
    /// Индекс соответствует положению текущего значения в списке возможных значений possible_values
    pub fn get_parameter_index_using_value(&self, key: &str) -> Result<u8, String> {
        let parameters_list = self.get_parameters_names()?;
        if parameters_list.contains(&key.to_owned()) {
            let parameters_possible_values_list = self.get_parameter_possible_values(key)?;
            let parameter_current_value = self.get_parameter_value(key)?;

            if let Some(index) = parameters_possible_values_list
                .iter()
                .position(|value| *value == parameter_current_value)
            {
                return Ok(index as u8);
            }
        }
        Err("Unable to get parameter value using index".to_string())
    }

    /// ## Присваивание параметру key значения, соответствующего index
    /// Индекс соответствует  положению  значения в списке возможных значений possible_values
    pub fn set_parameter_value_using_index(&mut self, key: &str, index: u8) -> Result<(), String> {
        let parameters_list = self.get_parameters_names()?;
        if parameters_list.contains(&key.to_owned()) {
            let parameters_possible_values_list = self.get_parameter_possible_values(key)?;
            if let Some(value) = parameters_possible_values_list.get(index as usize) {
                return self.set_parameter_value(key, value.clone());
            }
        }
        Err("Unable to get parameter value using index".to_string())
    }
}

/// Структура, описывающая параметр
#[derive(Debug, Clone)]
struct Parameter {
    /// Описание параметра (для использования в меню)
    description: String,
    /// Текущее значение параметра
    value: String,
    /// Список допустимых значений параметра
    possible_values: Vec<String>,
}

impl Parameter {
    /// Получение имени параметра (для использования в меню)
    fn get_description(&self) -> String {
        self.description.clone()
    }

    /// Получение текущего значения параметра
    fn get_value(&self) -> String {
        self.value.clone()
    }

    /// Установка текущего значения параметра
    fn set_value(&mut self, value: String) -> Result<(), String> {
        if !self.possible_values.contains(&value) {
            return Err(format!(
                "Parameter {} cannot be set to {}!",
                self.description, value
            ));
        }
        self.value = value;
        Ok(())
    }

    /// Получение списка возможных значений параметра
    fn get_possible_values(&self) -> Vec<String> {
        self.possible_values.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_io_operations() {
        let mut device_config =
            DeviceConfig::create_parameter_list("examples/simple_config.toml").unwrap();

        let names = device_config.get_parameters_names().unwrap();

        assert!(names.contains(&"groupnumber".to_string()));

        assert!(
            device_config
                .set_parameter_value("groupnumber", "19".to_string())
                .is_err()
        );

        assert!(
            device_config
                .set_parameter_value("groupnumber", "11".to_string())
                .is_ok()
        );

        assert_eq!(
            device_config.get_parameter_value("groupnumber").unwrap(),
            "11"
        );
    }

    #[test]
    fn test_parameter_saving() {
        let mut device_config =
            DeviceConfig::create_parameter_list("examples/simple_config.toml").unwrap();

        let names = device_config.get_parameters_names().unwrap();

        assert!(names.contains(&"soundvolume".to_string()));

        assert!(
            device_config
                .set_parameter_value("soundvolume", "100".to_string())
                .is_err()
        );

        assert!(
            device_config
                .set_parameter_value("soundvolume", "100%".to_string())
                .is_ok()
        );

        assert!(device_config.save_parameters_values().is_ok());

        let device_config =
            DeviceConfig::create_parameter_list("examples/simple_config.toml").unwrap();

        assert_eq!(
            device_config.get_parameter_value("soundvolume").unwrap(),
            "100%"
        );
    }

    #[test]
    fn test_get_parameter_index_using_value() {
        let mut device_config =
            DeviceConfig::create_parameter_list("examples/simple_config.toml").unwrap();

        for index in 0..device_config
            .get_parameter_possible_values("groupnumber")
            .unwrap()
            .len()
        {
            device_config
                .set_parameter_value("groupnumber", index.to_string())
                .unwrap();
            assert_eq!(
                device_config
                    .get_parameter_index_using_value("groupnumber")
                    .unwrap(),
                index as u8
            );
        }

        let capacity_values = device_config
            .get_parameter_possible_values("loadcapacity")
            .unwrap();

        for index in 0..capacity_values.len() {
            device_config
                .set_parameter_value("loadcapacity", capacity_values[index].clone())
                .unwrap();
            assert_eq!(
                device_config
                    .get_parameter_index_using_value("loadcapacity")
                    .unwrap(),
                index as u8
            );
        }
    }

    #[test]
    fn test_set_parameter_value_using_index() {
        let mut device_config =
            DeviceConfig::create_parameter_list("examples/simple_config.toml").unwrap();

        for index in 0..device_config
            .get_parameter_possible_values("groupnumber")
            .unwrap()
            .len()
        {
            device_config
                .set_parameter_value_using_index("groupnumber", index as u8)
                .unwrap();
            assert_eq!(
                device_config.get_parameter_value("groupnumber").unwrap(),
                index.to_string()
            );
        }

        for index in 0..device_config
            .get_parameter_possible_values("loadcapacity")
            .unwrap()
            .len()
        {
            device_config
                .set_parameter_value_using_index("loadcapacity", index as u8)
                .unwrap();
            assert_eq!(
                device_config.get_parameter_value("loadcapacity").unwrap(),
                device_config
                    .get_parameter_possible_values("loadcapacity")
                    .unwrap()[index]
            );
        }
    }
}
