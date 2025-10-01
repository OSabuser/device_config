use toml_edit::{DocumentMut, Item, Value};

#[derive(Debug, Clone)]
/// Структура для работы с TOML-конфигом
pub(crate) struct TomlScheme {
    path_to_scheme: String,
    document: DocumentMut,
    tables: Vec<String>,
}

impl TomlScheme {
    pub(crate) fn new(path_to_scheme: &str) -> Result<Self, String> {
        let toml_str = std::fs::read_to_string(path_to_scheme).map_err(|e| e.to_string())?;
        let doc = toml_str.parse::<DocumentMut>().map_err(|e| e.to_string())?;

        let table = doc.as_table();

        // Получение списка вложенных в файл таблиц-параметров
        let sub_tables: Vec<String> = table
            .iter()
            .filter_map(|(k, v)| {
                if v.is_table() {
                    Some(k.to_string())
                } else {
                    None
                }
            })
            .collect();

        if sub_tables.is_empty() {
            return Err("Parameters list is empty".to_string());
        }

        Ok(TomlScheme {
            path_to_scheme: path_to_scheme.to_string(),
            document: doc,
            tables: sub_tables,
        })
    }

    /// Получение пути к конфиг-файлу
    pub(crate) fn get_path_to_scheme_file(&self) -> String {
        self.path_to_scheme.clone()
    }

    /// Получение списка вложенных в файл параметров (TOML-таблицы)
    pub(crate) fn get_list_of_parameters(&self) -> Result<Vec<String>, String> {
        Ok(self.tables.clone())
    }

    /// Получение значения для `key`(подпараметр) у параметра `parameter_name`
    fn get_parameter_value(&self, parameter_name: &str, key: &str) -> Result<Value, String> {
        if let Some(table) = self.document.get(parameter_name) {
            if let Some(sub_table) = table.as_table() {
                if let Some(value) = sub_table.get(key) {
                    if let Some(value) = value.as_value() {
                        return Ok(value.clone());
                    }
                }
            }
        }

        return Err(format!("Unable to get {} value", key));
    }

    /// Получение строкового значения для `key`(подпараметр) у параметра `parameter_name`
    pub(crate) fn get_string_value(
        &self,
        parameter_name: &str,
        key: &str,
    ) -> Result<String, String> {
        let value = self.get_parameter_value(parameter_name, key)?;

        if let Some(value) = value.as_str() {
            return Ok(value.to_string());
        }
        Err(format!("Unable to get {} value as string", key))
    }

    /// Получение значения - массива строк для `key`(подпараметр) у параметра `parameter_name`
    pub(crate) fn get_array_value(
        &self,
        parameter_name: &str,
        key: &str,
    ) -> Result<Vec<String>, String> {
        let value = self.get_parameter_value(parameter_name, key)?;

        if let Some(array) = value.as_array() {
            return Ok(array
                .iter()
                .map(|x| x.as_str().expect("Unable to get array value").to_string())
                .collect());
        }
        Err(format!("Unable to get {} value as array", key))
    }

    /// Установка значения `value` для `key`(подпараметр) у параметра `parameter_name`
    pub(crate) fn set_parameter_value(
        &mut self,
        parameter_name: &str,
        key: &str,
        val: Item,
    ) -> Result<(), String> {
        if let Some(table) = self.document.get_mut(parameter_name) {
            if let Some(sub_table) = table.as_table() {
                if let Some(_) = sub_table.get(key) {
                    self.document[parameter_name][key] = val;

                    let new_toml = self.document.to_string();

                    std::fs::write(&self.path_to_scheme, new_toml)
                        .expect("Failed to write TOML file");

                    return Ok(());
                }
            }
        }

        return Err(format!(
            "Unable to set {}'s {} value to {:?}",
            parameter_name, key, val
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use toml_edit::value;
    #[test]
    fn test_get_list_of_parameters() {
        let toml_scheme = TomlScheme::new("examples/simple_config.toml").unwrap();
        let parameters = toml_scheme.get_list_of_parameters().unwrap();
        assert_eq!(parameters.len(), 4);
    }

    #[test]
    fn test_set_parameter_value() {
        let mut toml_scheme = TomlScheme::new("examples/simple_config.toml").unwrap();
        toml_scheme
            .set_parameter_value("groupnumber", "current", value("2:1234"))
            .unwrap();
        let value = toml_scheme
            .get_string_value("groupnumber", "current")
            .unwrap();
        assert_eq!(value.as_str(), "2:1234");
    }
}
