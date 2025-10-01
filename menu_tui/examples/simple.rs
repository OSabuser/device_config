use config_lib::device_config::DeviceConfig;
use menu_tui::menu_rendering::{MainMenuStates, show_main_dialog};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut device_config =
        DeviceConfig::create_parameter_list("menu_tui/examples/simple_config.toml")?;

    loop {
        match show_main_dialog(&mut device_config) {
            Ok(MainMenuStates::ConfigurationState) => {
                println!("Configuration state");
                continue;
            }
            Ok(MainMenuStates::ExitState) => {
                println!("The user has chosen to exit...");
                break;
            }
            Err(e) => {
                println!("Menu error: {}", e);
                break;
            }
        }
        
    }

    return Ok(());
}
