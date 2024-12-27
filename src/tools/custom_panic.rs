use std::{env::current_exe, panic, process::exit};
use native_dialog::{MessageDialog, MessageType};
use tokio::process::Command;



async fn restart_scroll() {
    let current_exe = match current_exe() {
        Ok(v) => v,
        Err(_) => {
            exit(1);
        },
    };
    match Command::new(current_exe).spawn() {
        Ok(v) => v,
        Err(_) => {
            exit(1);
        }
    };
}


pub fn set_custom_panic() {
    panic::set_hook(Box::new(|panic_info| {
        let message: &str = match panic_info.payload_as_str() {
            Some(s) => s,
            None => "Unknown error occured",
        };
        let location: String = match panic_info.location() {
            Some(l) => format!("File \"{}\" at {}:{}", l.file(), l.line(), l.column()),
            None => "Unknown location".to_string(),
        };
        let result = MessageDialog::new()
            .set_title("Scroll")
            .set_type(MessageType::Warning)
            .set_text(&format!(
                "An error has occured. Please contact a lead with the following debug trace:\n\nError: {message}\nLocation: {location}\n\n\nWould you like to restart Scroll?"))
            .show_confirm()
            .unwrap();
        if result {
	    tokio::spawn(async move {
		restart_scroll().await;  
            });
        }
        exit(1);
    }));
}
