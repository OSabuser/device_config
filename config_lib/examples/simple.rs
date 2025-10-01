use config_lib::device_config::DeviceConfig;
use rand::rng;
use rand::seq::IndexedRandom;

fn main() -> Result<(), String> {
    let mut config = DeviceConfig::create_parameter_list("config_lib/examples/simple_config.toml")?;

    let parameters = config.get_parameters_names()?;
    println!("Полученный список параметров: {:#?}", parameters);

    println!("Текущие значения параметров:\n");
    for parameter in &parameters {
        println!(
            "{} = {};\nдиапазон допустимых значений: {:?}\n",
            config.get_parameter_description(&parameter)?,
            config.get_parameter_value(&parameter)?,
            config.get_parameter_possible_values(&parameter)?
        );
    }

    let mut rng = rng();

    // Установка случайных значений для параметров в пределах допустимой области
    for parameter in &parameters {
        let random_value = config
            .get_parameter_possible_values(&parameter)?
            .choose(&mut rng)
            .unwrap()
            .clone();
        config.set_parameter_value(&parameter, random_value)?;
    }

    println!("Значения параметров после изменений:\n");
    for parameter in &parameters {
        println!(
            "{} = {};\nдиапазон допустимых значений: {:?}\n",
            config.get_parameter_description(&parameter)?,
            config.get_parameter_value(&parameter)?,
            config.get_parameter_possible_values(&parameter)?
        );
    }

    Ok(())
}
