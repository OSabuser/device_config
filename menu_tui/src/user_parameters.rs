use config_lib::device_config::DeviceConfig;

/// ### Структура для хранения пользовательских параметров
/// Дублирует данные из экземпляра `DeviceConfig` (пара ключ-значение из `HashMap<String, Parameter>`)
#[derive(Clone, Debug)]
pub struct Parameter {
    /// Имя параметра (совпадает с ключом в `HashMap<String, Parameter>`)
    pub key: String,
    /// Описание параметра (используется для отображения в интерфейсе)
    pub description: String,
    /// Возможные значения параметра
    pub options: Vec<String>,
    /// Текущее значение параметра
    pub selected_value: String,
}
impl Parameter {
    fn new(key: String, description: String, options: Vec<String>, selected: String) -> Self {
        Parameter {
            key,
            description,
            options,
            selected_value: selected,
        }
    }
}

/// Конфигурация устройства
/// Представляет собой набор  `Parameter`, дублирует данные из экземпляра `DeviceConfig`
#[derive(Clone, Debug)]
pub struct DeviceParameters {
    pub parameters: Vec<Parameter>,
}

impl Default for DeviceParameters {
    fn default() -> Self {
        DeviceParameters::new()
    }
}

impl DeviceParameters {
    pub fn new() -> Self {
        DeviceParameters {
            parameters: Vec::new(),
        }
    }

    /// Заполнение списка пользовательских параметров данными из `DeviceConfig`
    pub fn load_user_config(&mut self, parameters_schema: &DeviceConfig) -> Result<(), String> {
        for parameter_key in parameters_schema.get_parameters_names()? {
            let parameter_values = parameters_schema
                .get_parameter_possible_values(&parameter_key)?
                .clone();
            let parameter_desc = parameters_schema.get_parameter_description(&parameter_key)?;
            let parameter_value = parameters_schema
                .get_parameter_value(&parameter_key)?
                .clone();

            self.add_parameter(Parameter::new(
                parameter_key,
                parameter_desc,
                parameter_values,
                parameter_value,
            ));
        }
        Ok(())
    }

    /// Обновление конфигурации `DeviceConfig` по данным из `DeviceParameters`
    pub fn update_user_config(&self, parameters_schema: &mut DeviceConfig) -> Result<(), String> {
        for parameter in self.parameters.iter() {
            let value = parameter.selected_value.clone();
            parameters_schema.set_parameter_value(&parameter.key, value)?;
        }

        Ok(())
    }

    /// Добавление нового параметра
    fn add_parameter(&mut self, param: Parameter) {
        self.parameters.push(param);
    }

    /// Обновление значения параметра
    pub fn update_parameter(&mut self, key: &str, value: String) {
        if let Some(param) = self.parameters.iter_mut().find(|p| p.key == key) {
            param.selected_value = value;
        }
    }
}
